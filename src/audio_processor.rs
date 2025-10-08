use arrayvec::ArrayVec;
use clack_extensions::params::PluginAudioProcessorParams;
use clack_plugin::{
    events::{io::InputEventsIter, spaces::CoreEventSpace},
    host::HostAudioProcessorHandle,
    plugin::{PluginAudioProcessor, PluginError},
    prelude::{InputEvents, OutputEvents},
    process::{Audio, Events, PluginAudioConfiguration, Process, ProcessStatus},
};
use rand::{SeedableRng, rngs::SmallRng};

use crate::{
    consts::{
        KEYS_NR, MAX_NOTES_NR, MIDI_CC, MIDI_CC_ALL_NOTES_OFF, MIDI_CC_ALL_SOUNDS_OFF, MIDI_OFF,
        MIDI_ON, OSC_NR, PARAMETER_LEVEL_1, PARAMETER_LEVEL_3,
    },
    key::{Key, Keys, NoteData},
    main_thread::Fox3oscMainThread,
    shared::Fox3oscShared,
};

pub struct Fox3oscAudioProcessor<'a> {
    note_data: ArrayVec<NoteData, MAX_NOTES_NR>,
    keys: Keys,
    rng: SmallRng,
    shared: &'a Fox3oscShared,
}

impl Fox3oscAudioProcessor<'_> {
    fn process_cc_event(&mut self, midi_event: [u8; 3]) {
        let cc_nr = midi_event[1];
        match cc_nr {
            MIDI_CC_ALL_SOUNDS_OFF => self.keys.for_each(Key::end),
            MIDI_CC_ALL_NOTES_OFF => self.keys.for_each(Key::release),
            _ => {}
        }
    }

    fn process_events(&mut self, events: InputEventsIter) -> Result<(), PluginError> {
        for event in events {
            // Handle a parameter event
            if let Some(param_id) = self.shared.process_param_event(event)? {
                if matches!(param_id, PARAMETER_LEVEL_1..=PARAMETER_LEVEL_3) {
                    let osc = (param_id - PARAMETER_LEVEL_1) as usize;
                    let level = self.shared.get_levels()?[osc];

                    self.keys.for_each(move |key| {
                        key.set_level(level, osc);
                    });
                }

                continue;
            }

            // Handle a MIDI event
            let Some(CoreEventSpace::Midi(midi_event)) = event.as_core_event() else {
                continue;
            };

            let midi_event = midi_event.data();
            let midi_msg = midi_event[0] & 0xF0;
            match midi_msg {
                MIDI_ON => {
                    let note = midi_event[1] as usize % KEYS_NR;
                    let velocity = midi_event[2];

                    self.keys.on(note, velocity, self.shared, &mut self.rng)?;
                }
                MIDI_OFF => {
                    let note = midi_event[1] as usize % KEYS_NR;
                    self.keys.release(note);
                }
                MIDI_CC => self.process_cc_event(midi_event),
                _ => {}
            }
        }

        Ok(())
    }
}

impl<'a> PluginAudioProcessor<'a, Fox3oscShared, Fox3oscMainThread<'a>>
    for Fox3oscAudioProcessor<'a>
{
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut Fox3oscMainThread,
        shared: &'a Fox3oscShared,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        let sample_rate = audio_config.sample_rate as f32;
        let note_data = ArrayVec::from_iter((0..shared.notes_nr).map(|note| {
            NoteData::new(
                sample_rate,
                (note as f32) - shared.pitch_amount as f32,
                shared.n_tet,
            )
        }));

        Ok(Self {
            shared,
            note_data,
            rng: SmallRng::seed_from_u64(0xB00B5),
            keys: Keys::new(sample_rate),
        })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        let mut output_port = audio
            .output_port(0)
            .ok_or(PluginError::Message("No output port"))?;

        let mut output_channels = output_port
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Output is not f32"))?;

        let output = output_channels
            .channel_mut(0)
            .ok_or(PluginError::Message("Output channel 0 not found"))?;

        let mut status = ProcessStatus::Sleep;
        for batch in events.input.batch() {
            self.process_events(batch.events())?;

            let levels = self.shared.get_levels()?;
            let oscs: ArrayVec<usize, OSC_NR> = levels
                .into_iter()
                .enumerate()
                .filter_map(|(osc, level)| if level > 0.0 { Some(osc) } else { None })
                .collect();

            let pitch = self.shared.get_pitch()?;

            output[batch.sample_bounds()].fill(0.0);
            self.keys.for_each(|key| {
                status = ProcessStatus::Continue;
                key.process(
                    &mut output[batch.sample_bounds()],
                    pitch.map(|pitch| pitch as usize),
                    &mut self.rng,
                    &oscs,
                    &self.note_data,
                );
            });
        }

        Ok(status)
    }

    fn reset(&mut self) {
        self.keys.for_each(Key::end);
    }
}

impl PluginAudioProcessorParams for Fox3oscAudioProcessor<'_> {
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
        for event in input_parameter_changes {
            self.shared.process_param_event(event).unwrap();
        }
    }
}
