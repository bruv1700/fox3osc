# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Changelog file
- Parameters to change the pitch of each oscillator (-2 to +2 octaves)
- [15-TET], [17-TET], [19-TET] [22-TET] [23-TET] and [24-TET] support

[15-TET]: https://en.wikipedia.org/wiki/15_equal_temperament
[17-TET]: https://en.wikipedia.org/wiki/17_equal_temperament 
[19-TET]: https://en.wikipedia.org/wiki/19_equal_temperament
[22-TET]: https://en.wikipedia.org/wiki/22_equal_temperament
[23-TET]: https://en.wikipedia.org/wiki/23_equal_temperament
[24-TET]: https://en.wikipedia.org/wiki/Quarter_tone

## [0.1.0] 2025-09-14

### Added

- 3 oscillators
- Basic ADSR *(Attack, Decay, Sustain, Release)*
- 8 oscillator waveform types:
  - *Sine, Triangle, Square, Saw, Noise*: The basic.
  - *Sploinky, Skloinky*: Malformed square and saw waveforms, respectively. Very cute!!
  - *Random*: Chooses a random waveform out of the 7 previous for each key pressed.
- High quality processing toggle. Applies for triangle, square and saw waveforms. Low quality versions produce aliasing, high quality ones don't. But CPU usage is more conservative with the former.
- Option to use oscillator 3 as a modulator for oscillator 1. There are 2 modulation types:
  - *Phase*: Phase modulation (aka FM)
  - *Evil*: Weird f-ed up phase modulation. Modulating signal's phase does not increment each sample and is constant to the sample increment amount. ADSR and velocity filters are also applied to the modulating signal, so mess around with the ADSR. Intended for experimental sound design.

[Unreleased]: https://github.com/bruv1700/fox3osc/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/bruv1700/fox3osc/releases/tag/v0.1.0
