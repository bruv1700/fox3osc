use std::{
    ffi::c_int,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use clack_plugin::{
    events::{UnknownEvent, spaces::CoreEventSpace},
    plugin::{PluginError, PluginShared},
};

use crate::consts::{
    OSC_NR, PARAMETER_ATTACK, PARAMETER_DECAY, PARAMETER_HQ_1, PARAMETER_HQ_2, PARAMETER_HQ_3,
    PARAMETER_LEVEL_1, PARAMETER_LEVEL_2, PARAMETER_LEVEL_3, PARAMETER_MODULATION,
    PARAMETER_PITCH_1, PARAMETER_PITCH_2, PARAMETER_PITCH_3, PARAMETER_RELEASE, PARAMETER_SUSTAIN,
    PARAMETER_WAVEFORM_1, PARAMETER_WAVEFORM_2, PARAMETER_WAVEFORM_3,
};

#[derive(Clone, Copy)]
pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for Envelope {
    /// The default envelope shape. A 10 ms attack, 80% sustain and 100 ms decay and release.
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.8,
            release: 0.1,
        }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub enum Waveform {
    #[default]
    Sine,
    Triangle,
    Square,
    Saw,
    Noise,
    /// Very sploinky! :3
    Sploinky,
    /// Very skloinky! >_<
    Skloinky,
    /// Randomly chooses a different waveform for every note pressed.
    Random,
}

impl Waveform {
    pub const fn as_str(self) -> &'static str {
        match self {
            Waveform::Sine => "Sine",
            Waveform::Triangle => "Triangle",
            Waveform::Square => "Square",
            Waveform::Saw => "Saw",
            Waveform::Noise => "Noise",
            Waveform::Sploinky => "Sploinky",
            Waveform::Skloinky => "Skloinky",
            Waveform::Random => "Random",
        }
    }
}

impl From<Waveform> for f64 {
    fn from(waveform: Waveform) -> Self {
        waveform as c_int as f64
    }
}

impl From<f64> for Waveform {
    fn from(clap_value: f64) -> Self {
        debug_assert!(clap_value as c_int <= Waveform::Random as c_int);

        // SAFETY:
        // Waveform is #[repr(C)] which guarantees it being the same size and alignement as a c_int.
        unsafe { std::mem::transmute::<c_int, Self>(clap_value as c_int) }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub enum Modulation {
    #[default]
    None,
    Phase,
    Evil,
}

impl Modulation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Modulation::None => "None",
            Modulation::Phase => "Phase",
            Modulation::Evil => "Evil",
        }
    }
}

impl From<Modulation> for f64 {
    fn from(waveform: Modulation) -> Self {
        waveform as c_int as f64
    }
}

impl From<f64> for Modulation {
    fn from(clap_value: f64) -> Self {
        debug_assert!(clap_value as c_int <= Modulation::Evil as c_int);

        // SAFETY:
        // Modulation is #[repr(C)] which guarantees it being the same size and alignement as a c_int.
        unsafe { std::mem::transmute::<c_int, Self>(clap_value as c_int) }
    }
}

pub struct Fox3oscShared {
    envelope: RwLock<Envelope>,
    waveform: RwLock<[Waveform; OSC_NR]>,
    levels: RwLock<[f32; OSC_NR]>,
    hq: RwLock<[bool; OSC_NR]>,
    modulation: RwLock<Modulation>,
    pitch: RwLock<[f64; OSC_NR]>,
}

impl Default for Fox3oscShared {
    fn default() -> Self {
        Self {
            envelope: Default::default(),
            waveform: Default::default(),
            modulation: Default::default(),
            levels: RwLock::new([1.0, 0.0, 0.0]),
            hq: RwLock::new([true; OSC_NR]),
            pitch: RwLock::new([24.0; OSC_NR]),
        }
    }
}

impl PluginShared<'_> for Fox3oscShared {}

impl Fox3oscShared {
    const PARAMETER_READ_ERR: PluginError =
        PluginError::Message("Failed to acquire parameter read lock");

    const PARAMETER_WRITE_ERR: PluginError =
        PluginError::Message("Failed to acquire parameter read lock");

    /// Process a potential parameter event. Returns `false` if event is not a parameter event, otherwise
    /// `true`. Returns `Err` if it fails to aquire a parameter write lock.
    pub fn process_param_event(&self, event: &UnknownEvent) -> Result<bool, PluginError> {
        if let Some(CoreEventSpace::ParamValue(event)) = event.as_core_event() {
            let mut envelope = self.get_envelope_mut()?;
            let mut waveforms = self.get_waveforms_mut()?;
            let mut levels = self.get_levels_mut()?;
            let mut hq = self.get_hq_mut()?;
            let mut modulation = self.get_modulation_mut()?;
            let mut pitch = self.get_pitch_mut()?;

            match event.param_id().map(|x| x.into()) {
                Some(PARAMETER_ATTACK) => envelope.attack = event.value() as f32,
                Some(PARAMETER_DECAY) => envelope.decay = event.value() as f32,
                Some(PARAMETER_SUSTAIN) => envelope.sustain = event.value() as f32,
                Some(PARAMETER_RELEASE) => envelope.release = event.value() as f32,
                Some(PARAMETER_WAVEFORM_1) => waveforms[0] = event.value().into(),
                Some(PARAMETER_WAVEFORM_2) => waveforms[1] = event.value().into(),
                Some(PARAMETER_WAVEFORM_3) => waveforms[2] = event.value().into(),
                Some(PARAMETER_LEVEL_1) => levels[0] = event.value() as f32,
                Some(PARAMETER_LEVEL_2) => levels[1] = event.value() as f32,
                Some(PARAMETER_LEVEL_3) => levels[2] = event.value() as f32,
                Some(PARAMETER_HQ_1) => hq[0] = event.value() != 0.0,
                Some(PARAMETER_HQ_2) => hq[1] = event.value() != 0.0,
                Some(PARAMETER_HQ_3) => hq[2] = event.value() != 0.0,
                Some(PARAMETER_MODULATION) => *modulation = event.value().into(),
                Some(PARAMETER_PITCH_1) => pitch[0] = event.value(),
                Some(PARAMETER_PITCH_2) => pitch[1] = event.value(),
                Some(PARAMETER_PITCH_3) => pitch[2] = event.value(),
                _ => {}
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_envelope(&self) -> Result<RwLockReadGuard<'_, Envelope>, PluginError> {
        self.envelope.read().or(Err(Self::PARAMETER_READ_ERR))
    }

    pub fn get_envelope_mut(&self) -> Result<RwLockWriteGuard<'_, Envelope>, PluginError> {
        self.envelope.write().or(Err(Self::PARAMETER_WRITE_ERR))
    }

    pub fn get_waveforms(&self) -> Result<RwLockReadGuard<'_, [Waveform; OSC_NR]>, PluginError> {
        self.waveform.read().or(Err(Self::PARAMETER_READ_ERR))
    }

    pub fn get_waveforms_mut(
        &self,
    ) -> Result<RwLockWriteGuard<'_, [Waveform; OSC_NR]>, PluginError> {
        self.waveform.write().or(Err(Self::PARAMETER_WRITE_ERR))
    }

    pub fn get_levels(&self) -> Result<RwLockReadGuard<'_, [f32; OSC_NR]>, PluginError> {
        self.levels.read().or(Err(Self::PARAMETER_READ_ERR))
    }

    pub fn get_levels_mut(&self) -> Result<RwLockWriteGuard<'_, [f32; OSC_NR]>, PluginError> {
        self.levels.write().or(Err(Self::PARAMETER_WRITE_ERR))
    }

    pub fn get_hq(&self) -> Result<RwLockReadGuard<'_, [bool; OSC_NR]>, PluginError> {
        self.hq.read().or(Err(Self::PARAMETER_READ_ERR))
    }

    pub fn get_hq_mut(&self) -> Result<RwLockWriteGuard<'_, [bool; OSC_NR]>, PluginError> {
        self.hq.write().or(Err(Self::PARAMETER_WRITE_ERR))
    }

    pub fn get_modulation(&self) -> Result<RwLockReadGuard<'_, Modulation>, PluginError> {
        self.modulation.read().or(Err(Self::PARAMETER_READ_ERR))
    }

    pub fn get_modulation_mut(&self) -> Result<RwLockWriteGuard<'_, Modulation>, PluginError> {
        self.modulation.write().or(Err(Self::PARAMETER_WRITE_ERR))
    }

    pub fn get_pitch(&self) -> Result<RwLockReadGuard<'_, [f64; OSC_NR]>, PluginError> {
        self.pitch.read().or(Err(Self::PARAMETER_READ_ERR))
    }

    pub fn get_pitch_mut(&self) -> Result<RwLockWriteGuard<'_, [f64; OSC_NR]>, PluginError> {
        self.pitch.write().or(Err(Self::PARAMETER_WRITE_ERR))
    }
}
