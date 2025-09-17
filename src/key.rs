use std::f32::consts::TAU;

use arrayvec::ArrayVec;
use clack_plugin::plugin::PluginError;
use rand::{Rng, rngs::SmallRng};

use crate::{
    consts::{KEYS_NR, NOTES_NR, OSC_MOD, OSC_NR, PHASE_DRY, PHASE_NR},
    math,
    shared::{Envelope, Fox3oscShared, Modulation, Waveform},
};

#[derive(PartialEq)]
enum ADSRState {
    Ended,
    Attack(f32),
    Decay(f32),
    Sustain,
    Release(f32),
}

struct ADSR {
    state: ADSRState,
    attack_samples: f32,
    decay_samples: f32,
    sustain: f32,
    release_samples: f32,
    /// The current amplitude of the ADSR when it's in the `Attack` or `Decay` states. This is for
    /// smoothly transitioning to the `Release` states from those.
    ad_level: f32,
    /// The current amplitude of the ADSR when it's in the `Decay` or `Release` states. This is for
    /// smoothly transitioning to the `Attack` states from those.
    r_level: f32,
}

impl ADSR {
    /// Resets (or creates) the ADSR to an uninitialized state. This will set the ADSR state to `Ended`.
    pub fn reset() -> Self {
        ADSR {
            state: ADSRState::Ended,
            attack_samples: 0.0,
            decay_samples: 0.0,
            sustain: 0.0,
            release_samples: 0.0,
            ad_level: 0.0,
            r_level: 0.0,
        }
    }

    /// Initializes the ADSR with an envelope. This will set the ADSR state to `Attack`.
    pub fn on(&mut self, envelope: Envelope, sample_rate: f32) {
        if !matches!(self.state, ADSRState::Attack(_)) {
            self.state = ADSRState::Attack(0.0);
        }

        self.attack_samples = envelope.attack * sample_rate;
        self.decay_samples = envelope.decay * sample_rate;
        self.sustain = envelope.sustain;
        self.release_samples = envelope.release * sample_rate;
    }

    /// Processes and updates the ADSR state. This will return amplitude (0.0 to 1.0) accordingly.
    pub fn process(&mut self) -> f32 {
        match self.state {
            ADSRState::Attack(sample) => {
                self.state = if sample >= self.attack_samples {
                    ADSRState::Decay(0.0)
                } else {
                    ADSRState::Attack(sample + 1.0)
                };

                self.ad_level = sample / self.attack_samples;
                self.ad_level + self.r_level
            }
            ADSRState::Decay(sample) => {
                self.state = if sample >= self.decay_samples {
                    ADSRState::Sustain
                } else {
                    ADSRState::Decay(sample + 1.0)
                };

                self.ad_level = 1.0 - (1.0 - self.sustain) * (sample / self.decay_samples);
                self.r_level = self.ad_level;
                self.ad_level
            }
            ADSRState::Sustain => self.sustain,
            ADSRState::Release(sample) => {
                if sample >= self.release_samples {
                    *self = Self::reset();
                    0.0
                } else {
                    self.state = ADSRState::Release(sample + 1.0);
                    self.r_level = self.ad_level * (1.0 - sample / self.release_samples);
                    self.r_level
                }
            }
            ADSRState::Ended => 0.0,
        }
        .clamp(0.0, 1.0)
    }
}

/// Represents a recursive (state-tracking) DC blocker filter. Specifically, the difference equation:
///
/// `y(n) = x(n) - x(n - 1) + Ry(n - 1)`
///
/// # Resources
///
/// - https://github.com/PaulBatchelor/sndkit/blob/master/dsp/dcblocker.org
/// - https://ccrma.stanford.edu/~jos/filters/DC_Blocker.html
#[derive(Clone, Copy)]
struct DCBlocker {
    x: f32,
    y: f32,
}

impl DCBlocker {
    /// Resets (or creates) the filter to a non-recursed state.
    pub fn reset() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        const R: f32 = 0.995;

        self.y = sample - self.x + R * self.y;
        self.x = sample;
        self.y
    }
}

pub struct Key {
    /* --Per oscillator data-- */
    adsr: [ADSR; OSC_NR],
    /// Amplitude of currently proccessed sample
    phase: [f32; PHASE_NR],
    /// Used when processing sploinky and skloinky waveforms, and when doing phase and evil modulation.
    dc_blocker: [DCBlocker; OSC_NR],

    /// Function pointers per oscillator corresponding to their wave functions.
    process_waveform:
        [fn(&mut Self, rng: &mut SmallRng, osc: usize, transition_size: f32) -> f32; OSC_NR],

    /* --Per key data-- */
    modulation: Modulation,
    sample_rate: f32,
    note: usize,
    /// MIDI note velocity in amplitude (0.0..=1.0)
    velocity: f32,
}

impl Key {
    /// Creates a key in an uninitialized state. The frequency is calculated from `note`, which corresponds
    /// to a MIDI note. The sample increment and wave transition size is pre-calculated here. The ADSR
    /// is also set to an uninitialized state.
    fn new(sample_rate: f32, note: usize) -> Self {
        Self {
            sample_rate,
            note,
            adsr: std::array::from_fn(|_| ADSR::reset()),
            dc_blocker: std::array::from_fn(|_| DCBlocker::reset()),
            phase: [0.0; PHASE_NR],
            process_waveform: [Self::process_sine; OSC_NR],
            modulation: Modulation::None,
            velocity: 0.0,
        }
    }

    /// Initializes a key as pressed.
    fn on(
        &mut self,
        velocity: u8,
        shared: &Fox3oscShared,
        rng: &mut SmallRng,
    ) -> Result<(), PluginError> {
        if velocity == 0 {
            self.end();
            return Ok(());
        }

        let mut waveforms = *shared.get_waveforms()?;
        let envelope = *shared.get_envelope()?;
        let hq = *shared.get_hq()?;

        self.modulation = *shared.get_modulation()?;
        self.velocity = velocity as f32 / 127.0;

        for osc in 0..OSC_NR {
            self.adsr[osc].on(envelope, self.sample_rate);
            self.process_waveform[osc] = loop {
                match waveforms[osc] {
                    Waveform::Sine => break Self::process_sine,
                    Waveform::Noise => break Self::process_noise,
                    Waveform::Triangle if hq[osc] => break Self::process_triangle_hq,
                    Waveform::Triangle => break Self::process_triangle,
                    Waveform::Square if hq[osc] => break Self::process_square_hq,
                    Waveform::Square => break Self::process_square,
                    Waveform::Saw if hq[osc] => break Self::process_saw_hq,
                    Waveform::Saw => break Self::process_saw,
                    Waveform::Sploinky => break Self::process_sploinky,
                    Waveform::Skloinky => break Self::process_skloinky,
                    Waveform::Random => {
                        waveforms[osc] = rng.random_range(0.0..Waveform::Random.into()).into()
                    }
                }
            };
        }

        Ok(())
    }

    fn is_on(&self) -> bool {
        self.adsr.iter().all(|adsr| adsr.state != ADSRState::Ended)
    }

    pub fn end(&mut self) {
        self.phase = [0.0; PHASE_NR];
        for adsr in &mut self.adsr {
            *adsr = ADSR::reset();
        }

        for dc_blocker in &mut self.dc_blocker {
            *dc_blocker = DCBlocker::reset();
        }
    }

    pub fn release(&mut self) {
        for adsr in &mut self.adsr {
            adsr.state = ADSRState::Release(0.0);
        }
    }

    pub fn process(
        &mut self,
        output: &mut [f32],
        pitch: [usize; OSC_NR],
        levels: [f32; OSC_NR],
        rng: &mut SmallRng,
        oscs: &[usize],
        note_data: &[NoteData],
    ) {
        // We should never call this function on a key that isn't on
        debug_assert!(self.is_on());

        match self.modulation {
            Modulation::None => self.process_3sub(output, pitch, levels, rng, oscs, note_data),
            Modulation::Phase => self.process_1pm_1sub(output, pitch, levels, rng, oscs, note_data),
            Modulation::Evil => {
                self.process_1evil_1sub(output, pitch, levels, rng, oscs, note_data)
            }
        }
    }

    /// Regular subtractive synthesis.
    fn process_3sub(
        &mut self,
        output: &mut [f32],
        pitch: [usize; OSC_NR],
        levels: [f32; OSC_NR],
        rng: &mut SmallRng,
        oscs: &[usize],
        note_data: &[NoteData],
    ) {
        for &osc in oscs {
            let note_data = note_data[self.note + pitch[osc]];

            for sample in output.iter_mut() {
                *sample += (self.process_waveform[osc])(self, rng, osc, note_data.transition_size)
                    * self.velocity
                    * levels[osc]
                    * self.adsr[osc].process();

                self.phase[osc] = (self.phase[osc] + note_data.increment) % 1.0;
            }
        }
    }

    /// Oscillator 3's signal is used to modulate Oscillattor 1's phase. Adjusting Oscillator 3's level
    /// adjusts the mix of dry un-modulated signal and wet modulated signal that's output.
    fn process_1pm_1sub(
        &mut self,
        output: &mut [f32],
        pitch: [usize; OSC_NR],
        levels: [f32; OSC_NR],
        rng: &mut SmallRng,
        oscs: &[usize],
        note_data: &[NoteData],
    ) {
        for &osc in oscs {
            if osc == 0 {
                let osc1_note_data = note_data[self.note + pitch[osc]];
                let mod_osc_note_data = note_data[self.note + pitch[OSC_MOD]];

                for sample in output.iter_mut() {
                    /// Amount by which to scale down the PM signal's amplitude.
                    ///
                    /// I want the PM signal to be scaled down to 48% of the maximum amplitude because
                    /// modulating the Osc 1 signal with a higher amplitude than that creates very
                    /// nasty aliasing.
                    const OSC_MOD_LEVEL_MODIFIER: f32 = 100.0 / 48.0;

                    // We are using the ADSR signal in multiple points here so we're processing it
                    // only once here and reusing it where needed.
                    let osc1_adsr = self.adsr[osc].process();
                    let sample_dc = (self.process_waveform[osc])(
                        self,
                        rng,
                        osc,
                        osc1_note_data.transition_size,
                    ) * self.velocity
                        * levels[osc]
                        * osc1_adsr;

                    *sample += self.dc_blocker[osc].process(sample_dc);
                    self.phase[osc] = ((self.phase[osc]
                        + (self.process_waveform[OSC_MOD])(
                            self,
                            rng,
                            OSC_MOD,
                            mod_osc_note_data.transition_size,
                        ))
                        * (levels[OSC_MOD] / OSC_MOD_LEVEL_MODIFIER))
                        % 1.0;

                    self.phase[OSC_MOD] = (self.phase[OSC_MOD] + mod_osc_note_data.increment) % 1.0;
                    self.phase[PHASE_DRY] =
                        (self.phase[PHASE_DRY] + osc1_note_data.increment) % 1.0;

                    self.phase.swap(0, PHASE_DRY);
                    *sample += (self.process_waveform[osc])(
                        self,
                        rng,
                        osc,
                        osc1_note_data.transition_size,
                    ) * self.velocity
                        * (levels[osc] - (levels[osc] * levels[OSC_MOD]) / OSC_MOD_LEVEL_MODIFIER)
                        * osc1_adsr;

                    self.phase.swap(0, PHASE_DRY);
                }
            } else if osc == 1 {
                let note_data = note_data[self.note + pitch[osc]];

                for sample in output.iter_mut() {
                    *sample +=
                        (self.process_waveform[osc])(self, rng, osc, note_data.transition_size)
                            * self.velocity
                            * levels[osc]
                            * self.adsr[osc].process();
                    self.phase[osc] = (self.phase[osc] + note_data.increment) % 1.0;
                }
            }
        }
    }

    /// Oscillator 3's signal is filtered with its velocity and ADSR like in subtractive synthesis,
    /// but we're modulating oscillators 1's signal with it. Along with this, we don't increment the
    /// phase of oscillator 3's signal. It's set to a constant value of the sample increment (frequency /
    /// sample rate, `self.increment`). I suppose this may be considered "subtractive phase modulation
    /// without phase incrementing", but "evil modulation" was chosen because it sounds fun.
    ///
    /// This type of synthesis, along with sploinky and skloinky waveforms are the result of incorrect
    /// implementations, in this case, of just regular phase modulation.
    ///
    /// Unlike phase modulation, we don't mix any dry signal.
    fn process_1evil_1sub(
        &mut self,
        output: &mut [f32],
        pitch: [usize; OSC_NR],
        levels: [f32; OSC_NR],
        rng: &mut SmallRng,
        oscs: &[usize],
        note_data: &[NoteData],
    ) {
        for &osc in oscs {
            if osc == 0 {
                let osc1_note_data = note_data[self.note + pitch[osc]];
                let mod_note_data = note_data[self.note + pitch[OSC_MOD]];

                for sample in output.iter_mut() {
                    let sample_dc = (self.process_waveform[osc])(
                        self,
                        rng,
                        osc,
                        osc1_note_data.transition_size,
                    ) * self.velocity
                        * levels[osc]
                        * self.adsr[osc].process();

                    *sample += self.dc_blocker[osc].process(sample_dc);

                    self.phase[OSC_MOD] = osc1_note_data.increment;
                    self.phase[osc] = (self.phase[osc]
                        + ((self.process_waveform[OSC_MOD])(
                            self,
                            rng,
                            OSC_MOD,
                            mod_note_data.transition_size,
                        )) * self.velocity
                            * levels[OSC_MOD]
                            * self.adsr[OSC_MOD].process())
                        % 1.0;
                }
            } else if osc == 1 {
                let note_data = note_data[self.note + pitch[osc]];

                for sample in output.iter_mut() {
                    *sample +=
                        (self.process_waveform[osc])(self, rng, osc, note_data.transition_size)
                            * self.velocity
                            * levels[osc]
                            * self.adsr[osc].process();
                    self.phase[osc] = (self.phase[osc] + note_data.increment) % 1.0;
                }
            }
        }
    }

    /// A sine waveform.
    fn process_sine(&mut self, _rng: &mut SmallRng, osc: usize, _transition_size: f32) -> f32 {
        (self.phase[osc] * TAU).sin()
    }

    /// A noise waveform tsssssssssssshh.
    fn process_noise(&mut self, rng: &mut SmallRng, _osc: usize, _transition_size: f32) -> f32 {
        rng.random_range(-1.0..1.0)
    }

    fn process_triangle_hq(
        &mut self,
        _rng: &mut SmallRng,
        osc: usize,
        transition_size: f32,
    ) -> f32 {
        4.0 * math::integrate_square_wave((self.phase[osc] + 0.25).rem_euclid(1.0), transition_size)
            - 1.0
    }

    /// An naive aliasing triangle waveform.
    fn process_triangle(&mut self, _rng: &mut SmallRng, osc: usize, _transition_size: f32) -> f32 {
        let p = self.phase[osc] % 1.0;

        if p < 0.25 {
            4.0 * p
        } else if p < 0.75 {
            1.0 - 4.0 * (p - 0.25)
        } else {
            -1.0 + 4.0 * (p - 0.75)
        }
    }

    /// A polyblep square waveform.
    fn process_square_hq(&mut self, _rng: &mut SmallRng, osc: usize, transition_size: f32) -> f32 {
        let p = self.phase[osc] % 1.0;

        (if p < 0.5 { 1.0 } else { -1.0 })
            + math::polyblep((((self.phase[osc] + 0.5) % 1.0) - 0.5) / transition_size)
            - math::polyblep((p - 0.5) / transition_size)
    }

    /// An naive aliasing square waveform.
    fn process_square(&mut self, _rng: &mut SmallRng, osc: usize, _transition_size: f32) -> f32 {
        let p = self.phase[osc] % 1.0;
        if p < 0.5 { 1.0 } else { -1.0 }
    }

    /// A polyblep saw waveform.
    fn process_saw_hq(&mut self, _rng: &mut SmallRng, osc: usize, transition_size: f32) -> f32 {
        let p = self.phase[osc] % 1.0;

        2.0 * p - 1.0 - math::polyblep((((self.phase[osc] + 0.5) % 1.0) - 0.5) / transition_size)
    }

    /// An naive aliasing saw waveform.
    fn process_saw(&mut self, _rng: &mut SmallRng, osc: usize, _transition_size: f32) -> f32 {
        let p = self.phase[osc] % 1.0;

        2.0 * p - 1.0
    }

    /// A polyblep square waveform whose transition points have been calculated incorrectly. This
    /// as a result in a waveform that's still aliasing despite the bandlimiting, as it's happening
    /// at the wrong point in the wave. Name was chosen arbitrarilly because it sounds cute.
    ///
    /// Since the waveform generated by this is so incorrect, we apply a DC blocking filter.
    fn process_sploinky(&mut self, _rng: &mut SmallRng, osc: usize, transition_size: f32) -> f32 {
        let p = self.phase[osc] % 1.0;

        self.dc_blocker[osc].process(
            (if p < 0.5 { 1.0 } else { -1.0 }
                + math::polyblep(((self.phase[osc] + 0.5) % 0.5) - transition_size)
                - math::polyblep((p - 0.5) / transition_size))
                / 2.0,
        )
    }

    /// A polyblep saw waveform whose transition points have been calculated incorrectly. This
    /// as a result in a waveform that's still aliasing despite the bandlimiting, as it's happening
    /// at the wrong point in the wave. Name was chosen arbitrarilly because it sounds cute.
    ///
    /// Since the waveform generated by this is so incorrect, we apply a DC blocking filter.
    fn process_skloinky(&mut self, _rng: &mut SmallRng, osc: usize, transition_size: f32) -> f32 {
        let p = self.phase[osc] % 1.0;
        self.dc_blocker[osc].process(
            (2.0 * p - 1.0 - math::polyblep(((self.phase[osc] + 0.5) % 0.5) - transition_size))
                / 2.0,
        )
    }
}

#[derive(Clone, Copy)]
pub struct NoteData {
    pub increment: f32,
    pub transition_size: f32,
}

impl NoteData {
    pub fn new(sample_rate: f32, note: f32) -> Self {
        let frequency = 2.0f32.powf((note - 69.0) / 12.0) * 440.0;
        let increment = frequency / sample_rate;
        let transition_size = 2.0 / (sample_rate / frequency);

        Self {
            increment,
            transition_size,
        }
    }
}

pub struct Keys {
    alive_keys: ArrayVec<usize, KEYS_NR>,
    keys: [Key; KEYS_NR],
    note_data: [NoteData; NOTES_NR],
}

impl Keys {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            alive_keys: ArrayVec::new(),
            keys: std::array::from_fn(move |note| Key::new(sample_rate, note)),
            note_data: std::array::from_fn(move |note| {
                NoteData::new(sample_rate, (note as f32) - 24.0)
            }),
        }
    }

    pub fn on(
        &mut self,
        note: usize,
        velocity: u8,
        shared: &Fox3oscShared,
        rng: &mut SmallRng,
    ) -> Result<(), PluginError> {
        debug_assert!(note < KEYS_NR);

        self.keys[note].on(velocity, shared, rng)?;
        if !self.alive_keys.contains(&note) {
            // SAFETY:
            // We check both whether note is less than KEYS_NR as well as whether note is already in
            // the vector. Therefore, this will never push note if the capacity isn't sufficient.
            unsafe { self.alive_keys.push_unchecked(note) };
        }

        Ok(())
    }

    pub fn release(&mut self, note: usize) {
        self.keys[note].release();
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Key, &[NoteData]),
    {
        let mut i = 0;
        while i < self.alive_keys.len() {
            let key = &mut self.keys[self.alive_keys[i]];

            if key.is_on() {
                f(key, &self.note_data);
                i += 1;
            } else {
                key.end();
                self.alive_keys.remove(i);
            }
        }
    }
}
