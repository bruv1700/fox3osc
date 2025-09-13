use std::{
    ffi::CStr,
    io::{Read, Write},
};

use clack_extensions::{
    audio_ports::{AudioPortFlags, AudioPortInfo, AudioPortType, PluginAudioPortsImpl},
    note_ports::{NoteDialect, NoteDialects, NotePortInfo, PluginNotePortsImpl},
    params::{
        ParamDisplayWriter, ParamInfo, ParamInfoFlags, ParamInfoWriter, PluginMainThreadParams,
    },
    state::PluginStateImpl,
};
use clack_plugin::{
    prelude::*,
    stream::{InputStream, OutputStream},
};

use crate::{
    consts::{
        CLAP_PARAM_IS_ENUM, PARAMETER_ATTACK, PARAMETER_DECAY, PARAMETER_HQ_1, PARAMETER_HQ_2,
        PARAMETER_HQ_3, PARAMETER_LEVEL_1, PARAMETER_LEVEL_2, PARAMETER_LEVEL_3,
        PARAMETER_MODULATION, PARAMETER_NR, PARAMETER_RELEASE, PARAMETER_SUSTAIN,
        PARAMETER_WAVEFORM_1, PARAMETER_WAVEFORM_2, PARAMETER_WAVEFORM_3,
    },
    shared::{Envelope, Fox3oscShared, Modulation, Waveform},
};

pub struct Fox3oscMainThread<'a> {
    shared: &'a Fox3oscShared,
}

impl<'a> PluginMainThread<'a, Fox3oscShared> for Fox3oscMainThread<'a> {}

impl<'a> Fox3oscMainThread<'a> {
    pub fn new(shared: &'a Fox3oscShared) -> Self {
        Self { shared }
    }
}

impl PluginAudioPortsImpl for Fox3oscMainThread<'_> {
    fn count(&mut self, is_input: bool) -> u32 {
        if !is_input { 1 } else { 0 }
    }

    fn get(
        &mut self,
        index: u32,
        is_input: bool,
        writer: &mut clack_extensions::audio_ports::AudioPortInfoWriter,
    ) {
        if !is_input && index == 0 {
            writer.set(&AudioPortInfo {
                id: ClapId::new(1),
                name: b"main",
                channel_count: 1,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::MONO),
                in_place_pair: None,
            });
        }
    }
}

impl PluginNotePortsImpl for Fox3oscMainThread<'_> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input { 1 } else { 0 }
    }

    fn get(
        &mut self,
        index: u32,
        is_input: bool,
        writer: &mut clack_extensions::note_ports::NotePortInfoWriter,
    ) {
        if is_input && index == 0 {
            writer.set(&NotePortInfo {
                id: ClapId::new(1),
                name: b"main",
                preferred_dialect: Some(NoteDialect::Midi),
                supported_dialects: NoteDialects::MIDI,
            })
        }
    }
}

fn get_info_adsr(param_index: u32, info: &mut ParamInfoWriter) {
    if let Some((name, default)) = match param_index {
        PARAMETER_ATTACK => Some(("Attack", Envelope::default().attack)),
        PARAMETER_DECAY => Some(("Decay", Envelope::default().decay)),
        PARAMETER_SUSTAIN => Some(("Sustain", Envelope::default().sustain)),
        PARAMETER_RELEASE => Some(("Release", Envelope::default().release)),
        _ => None,
    } {
        info.set(&ParamInfo {
            id: param_index.into(),
            flags: ParamInfoFlags::IS_AUTOMATABLE,
            cookie: Default::default(),
            name: name.as_bytes(),
            module: b"",
            min_value: 0.0,
            max_value: 1.0,
            default_value: default as f64,
        });
    }
}

fn get_info_waveforms(param_index: u32, info: &mut ParamInfoWriter) {
    if let Some((name, default)) = match param_index {
        PARAMETER_WAVEFORM_1 => Some(("Osc 1 Waveform", Waveform::default())),
        PARAMETER_WAVEFORM_2 => Some(("Osc 2 Waveform", Waveform::default())),
        PARAMETER_WAVEFORM_3 => Some(("Osc 3 Waveform", Waveform::default())),
        _ => None,
    } {
        info.set(&ParamInfo {
            id: param_index.into(),
            flags: CLAP_PARAM_IS_ENUM | ParamInfoFlags::IS_STEPPED | ParamInfoFlags::IS_AUTOMATABLE,
            cookie: Default::default(),
            name: name.as_bytes(),
            module: b"",
            min_value: Waveform::Sine.into(),
            max_value: Waveform::Random.into(),
            default_value: default.into(),
        });
    }
}

fn get_info_levels(param_index: u32, info: &mut ParamInfoWriter) {
    if let Some((name, default)) = match param_index {
        PARAMETER_LEVEL_1 => Some(("Osc 1 Level", 1.0)),
        PARAMETER_LEVEL_2 => Some(("Osc 2 Level", 0.0)),
        PARAMETER_LEVEL_3 => Some(("Osc 3 Level", 0.0)),
        _ => None,
    } {
        info.set(&ParamInfo {
            id: param_index.into(),
            flags: ParamInfoFlags::IS_AUTOMATABLE,
            cookie: Default::default(),
            name: name.as_bytes(),
            module: b"",
            min_value: 0.0,
            max_value: 1.0,
            default_value: default,
        });
    }
}

fn get_info_hq(param_index: u32, info: &mut ParamInfoWriter) {
    if let Some((name, default)) = match param_index {
        PARAMETER_HQ_1 => Some(("Osc 1 HQ", true)),
        PARAMETER_HQ_2 => Some(("Osc 2 HQ", true)),
        PARAMETER_HQ_3 => Some(("Osc 3 HQ", true)),
        _ => None,
    } {
        info.set(&ParamInfo {
            id: param_index.into(),
            flags: CLAP_PARAM_IS_ENUM | ParamInfoFlags::IS_STEPPED | ParamInfoFlags::IS_AUTOMATABLE,
            cookie: Default::default(),
            name: name.as_bytes(),
            module: b"",
            min_value: 0.0,
            max_value: 1.0,
            default_value: default as u8 as f64,
        });
    }
}

fn get_info_modulation(param_index: u32, info: &mut ParamInfoWriter) {
    if let Some((name, default)) = match param_index {
        PARAMETER_MODULATION => Some(("Osc 3 -> Osc 1 Modulation", Modulation::default())),
        _ => None,
    } {
        info.set(&ParamInfo {
            id: param_index.into(),
            flags: CLAP_PARAM_IS_ENUM | ParamInfoFlags::IS_STEPPED | ParamInfoFlags::IS_AUTOMATABLE,
            cookie: Default::default(),
            name: name.as_bytes(),
            module: b"",
            min_value: Modulation::None.into(),
            max_value: Modulation::Evil.into(),
            default_value: default.into(),
        });
    }
}

impl PluginMainThreadParams for Fox3oscMainThread<'_> {
    /// Number of plugin parameters.
    fn count(&mut self) -> u32 {
        PARAMETER_NR
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        self::get_info_adsr(param_index, info);
        self::get_info_waveforms(param_index, info);
        self::get_info_levels(param_index, info);
        self::get_info_hq(param_index, info);
        self::get_info_modulation(param_index, info);
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        let envelope = self.shared.get_envelope().ok()?;
        let waveform = self.shared.get_waveforms().ok()?;
        let levels = self.shared.get_levels().ok()?;
        let hq = self.shared.get_hq().ok()?;
        let modulation = self.shared.get_modulation().ok()?;

        match param_id.into() {
            PARAMETER_ATTACK => Some(envelope.attack as f64),
            PARAMETER_DECAY => Some(envelope.decay as f64),
            PARAMETER_SUSTAIN => Some(envelope.sustain as f64),
            PARAMETER_RELEASE => Some(envelope.release as f64),
            PARAMETER_WAVEFORM_1 => Some((waveform[0]).into()),
            PARAMETER_WAVEFORM_2 => Some((waveform[1]).into()),
            PARAMETER_WAVEFORM_3 => Some((waveform[2]).into()),
            PARAMETER_LEVEL_1 => Some(levels[0] as f64),
            PARAMETER_LEVEL_2 => Some(levels[1] as f64),
            PARAMETER_LEVEL_3 => Some(levels[2] as f64),
            PARAMETER_HQ_1 => Some(hq[0] as u8 as f64),
            PARAMETER_HQ_2 => Some(hq[1] as u8 as f64),
            PARAMETER_HQ_3 => Some(hq[2] as u8 as f64),
            PARAMETER_MODULATION => Some((*modulation).into()),
            _ => None,
        }
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result {
        use std::fmt::Write;
        match param_id.into() {
            PARAMETER_ATTACK | PARAMETER_DECAY | PARAMETER_RELEASE => {
                write!(writer, "{:.2} s", value)
            }
            PARAMETER_SUSTAIN | PARAMETER_LEVEL_1..=PARAMETER_LEVEL_3 => {
                write!(writer, "{:.2} %", value * 100f64)
            }
            PARAMETER_WAVEFORM_1..=PARAMETER_WAVEFORM_3 => {
                write!(writer, "{}", Waveform::from(value).as_str())
            }
            PARAMETER_HQ_1..=PARAMETER_HQ_3 => {
                write!(writer, "{}", value != 0.0)
            }
            PARAMETER_MODULATION => {
                write!(writer, "{}", Modulation::from(value).as_str())
            }
            _ => Err(std::fmt::Error),
        }
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &CStr) -> Option<f64> {
        let input = text.to_str().ok()?;

        match param_id.get() {
            param_id @ (PARAMETER_ATTACK..=PARAMETER_RELEASE
            | PARAMETER_LEVEL_1..=PARAMETER_LEVEL_3) => {
                let scale = if matches!(
                    param_id,
                    PARAMETER_SUSTAIN | PARAMETER_LEVEL_1..=PARAMETER_LEVEL_3
                ) {
                    0.01
                } else {
                    1.0
                };

                let suffix_idx = input
                    .find(|c: char| !c.is_numeric() && c != '.' && c != ',')
                    .unwrap_or(input.len());

                input[..suffix_idx].parse().map(|v: f64| v * scale).ok()
            }
            PARAMETER_HQ_1..=PARAMETER_HQ_3 => Some(input.parse::<bool>().ok()? as u8 as f64),
            _ if input == Waveform::Sine.as_str() => Some(Waveform::Sine.into()),
            _ if input == Waveform::Triangle.as_str() => Some(Waveform::Triangle.into()),
            _ if input == Waveform::Square.as_str() => Some(Waveform::Square.into()),
            _ if input == Waveform::Saw.as_str() => Some(Waveform::Saw.into()),
            _ if input == Waveform::Noise.as_str() => Some(Waveform::Noise.into()),
            _ if input == Waveform::Sploinky.as_str() => Some(Waveform::Sploinky.into()),
            _ if input == Waveform::Skloinky.as_str() => Some(Waveform::Skloinky.into()),
            _ if input == Waveform::Random.as_str() => Some(Waveform::Random.into()),
            _ if input == Modulation::None.as_str() => Some(Modulation::None.into()),
            _ if input == Modulation::Phase.as_str() => Some(Modulation::Phase.into()),
            _ if input == Modulation::Evil.as_str() => Some(Modulation::Evil.into()),
            _ => None,
        }
    }

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

impl PluginStateImpl for Fox3oscMainThread<'_> {
    /// Save the plugin parameter state.
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        let envelope = self.shared.get_envelope()?;
        let waveforms = self.shared.get_waveforms()?;
        let levels = self.shared.get_levels()?;
        let hq = self.shared.get_hq()?;
        let modulation = self.shared.get_modulation()?;

        output.write_all(&envelope.attack.to_le_bytes())?;
        output.write_all(&envelope.decay.to_le_bytes())?;
        output.write_all(&envelope.sustain.to_le_bytes())?;
        output.write_all(&envelope.release.to_le_bytes())?;
        for &waveform in waveforms.iter() {
            output.write_all(&f64::from(waveform).to_le_bytes())?;
        }

        for &level in levels.iter() {
            output.write_all(&level.to_le_bytes())?;
        }

        for &hq in hq.iter() {
            output.write_all(&(hq as u32).to_le_bytes())?;
        }

        output.write_all(&f64::from(*modulation).to_le_bytes())?;
        Ok(())
    }

    /// Load the plugin parameter state.
    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError> {
        let mut envelope = self.shared.get_envelope_mut()?;
        let mut waveforms = self.shared.get_waveforms_mut()?;
        let mut levels = self.shared.get_levels_mut()?;
        let mut hq = self.shared.get_hq_mut()?;
        let mut modulation = self.shared.get_modulation_mut()?;

        let mut buf = [0; 4];
        input.read_exact(&mut buf)?;
        envelope.attack = f32::from_le_bytes(buf);
        input.read_exact(&mut buf)?;
        envelope.decay = f32::from_le_bytes(buf);
        input.read_exact(&mut buf)?;
        envelope.sustain = f32::from_le_bytes(buf);
        input.read_exact(&mut buf)?;
        envelope.release = f32::from_le_bytes(buf);

        let mut buf = [0; 8];
        for waveform in waveforms.iter_mut() {
            input.read_exact(&mut buf)?;
            *waveform = f64::from_le_bytes(buf).into();
        }

        let mut buf = [0; 4];
        for level in levels.iter_mut() {
            input.read_exact(&mut buf)?;
            *level = f32::from_le_bytes(buf);
        }

        for hq in hq.iter_mut() {
            input.read_exact(&mut buf)?;
            *hq = u32::from_le_bytes(buf) != 0;
        }

        let mut buf = [0; 8];
        input.read_exact(&mut buf)?;
        *modulation = f64::from_le_bytes(buf).into();

        Ok(())
    }
}
