use clack_extensions::params::ParamInfoFlags;

/// This parameter represents an enumerated value. If you set this flag, then you must set CLAP_PARAM_IS_STEPPED
/// too. All values from min to max must not have a blank value_to_text().
///
/// For some reason this does not exist in neither clack or the clap-sys build that clack uses. But
/// it's very much so a real parameter info flag that's documented in the CLAP API. See the [header
/// file] in which this is defined in.
///
/// [header file]: https://github.com/free-audio/clap/blob/69a69252fdd6ac1d06e246d9a04c0a89d9607a17/include/clap/ext/params.h#L199C1-L202C33
pub const CLAP_PARAM_IS_ENUM: ParamInfoFlags = ParamInfoFlags::from_bits_retain(1 << 16);

/// Number of MIDI notes and max polyphony of fox3osc
pub const KEYS_NR: usize = 128;
/// Number of oscillators *(The 3 in fox3osc)*
pub const OSC_NR: usize = 3;

pub const MIDI_ON: u8 = 0x90;
pub const MIDI_OFF: u8 = 0x80;
pub const MIDI_CC: u8 = 0xB0;

pub const MIDI_CC_ALL_SOUNDS_OFF: u8 = 0x78;
pub const MIDI_CC_ALL_NOTES_OFF: u8 = 0x7B;

pub const PARAMETER_ATTACK: u32 = 0;
pub const PARAMETER_DECAY: u32 = 1;
pub const PARAMETER_SUSTAIN: u32 = 2;
pub const PARAMETER_RELEASE: u32 = 3;
pub const PARAMETER_WAVEFORM_1: u32 = 4;
pub const PARAMETER_WAVEFORM_2: u32 = 5;
pub const PARAMETER_WAVEFORM_3: u32 = 6;
pub const PARAMETER_LEVEL_1: u32 = 7;
pub const PARAMETER_LEVEL_2: u32 = 8;
pub const PARAMETER_LEVEL_3: u32 = 9;
pub const PARAMETER_HQ_1: u32 = 10;
pub const PARAMETER_HQ_2: u32 = 11;
pub const PARAMETER_HQ_3: u32 = 12;
pub const PARAMETER_MODULATION: u32 = 13;
pub const PARAMETER_NR: u32 = 14;
