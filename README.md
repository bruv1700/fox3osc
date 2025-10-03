# fox3osc

Lightweight CLAP subtractive synthesizer with 3 oscillators and phase modulation.

# Features

- Basic ADSR *(Attack, Decay, Sustain, Release)*
- 8 oscillator waveform types:
  - *Sine, Triangle, Square, Saw, Noise*: The basic.
  - *Sploinky, Skloinky*: Malformed square and saw waveforms, respectively. Very cute!!
  - *Random*: Chooses a random waveform out of the 7 previous for each key pressed.
- High quality processing toggle. Applies for triangle, square and saw waveforms. Low quality versions produce aliasing, high quality ones don't. But CPU usage is more conservative with the former.
- Use oscillator 3 as a modulator for oscillator 1. There are 2 modulation types:
  - *Phase*: Phase modulation (aka FM)
  - *Evil*: Weird f-ed up phase modulation. Modulating signal's phase does not increment each sample and is constant to the sample increment amount. ADSR and velocity filters are also applied to the modulating signal, so mess around with the ADSR. Intended for experimental sound design.
- [15-TET], [17-TET], [19-TET] [22-TET] [23-TET] and [24-TET] support.

[15-TET]: https://en.wikipedia.org/wiki/15_equal_temperament
[17-TET]: https://en.wikipedia.org/wiki/17_equal_temperament 
[19-TET]: https://en.wikipedia.org/wiki/19_equal_temperament
[22-TET]: https://en.wikipedia.org/wiki/22_equal_temperament
[23-TET]: https://en.wikipedia.org/wiki/23_equal_temperament
[24-TET]: https://en.wikipedia.org/wiki/Quarter_tone

# Installation

I publish binaries for **windows** and **linux** on the releases section on github. Download and place fox3osc in your clap plugin folder. If your CPU isn't **x86_64**, or you're on neither OS download the source code and build it from source.

## Installation from source

To build fox3osc, you will need **[cargo]**.

Build the plugin with cargo (`cargo build --release`) and copy the built binary from `./target/release/` to your clap plugin folder. Make sure to rename the extension to `.clap`. On MacOS, you'll need to make an app bundle from scratch instead.

By default, cargo will build fox3osc with only 12-TET support. You can compile the plugin with support for microtones by specifying the supported scales in the `--features` argument (eg: `cargo build --features "15tet 19tet"`).

Alternatively, a **[justfile]** is provided to make installation super convenient. Just can be installed with cargo very easily with `cargo install just`. `just install` will then install the plugin to your user clap plugin folder (eg: `~/.clap` on linux).

Building the plugin with the justfile (Running `just`) uses the nightly version of rustc and cargo in order to agressively optimize the size of the final binary. It will download and install the `rust-src` rustup component for nightly if not present, which is needed for the `-Z build-std` feature on cargo to work. You can install the latest version of nightly rust with `rustup toolchain install nightly`.

Of course the nightly compiler isn't needed to build fox3osc, this is just so the final binary can be as small as possible.

**This is how I reccomend you build fox3osc:**
```shell
cargo install just  # In case you don't have just already installed
just                # Or alternatively "cargo build --release" if you don't care about having a smaller binary
just install
```

By default the justfile will compile the plugin with all the microtonal scales enabled. If you want to customize which scales you want specify whichever of the supported scales after `just build` (eg: `just build "15tet 19tet"`). You can build fox3osc without microtonal support with `just build " "` (The space inbetween the quotes is important).

[cargo]: https://doc.rust-lang.org/cargo/
[justfile]: https://just.systems/man/en/

# Resources

This is the first audio plugin I made, so I feel obligated to share the resources I studied in order to make this:

- *https://github.com/sjaehn/lv2tutorial:* Great introduction to DSP. Focuses on [LV2 plugins], but the knowledge gained from the series is transferable:
  - *https://www.youtube.com/playlist?list=PLkuRaNsK2AJ0D8uhRIjftgmqVW0yvDfMx*
- *https://github.com/Kwarf/crabhowler:* Kwarf's blog is a great introduction in my opinion to the [clack] API:
  - *https://kwarf.com/2024/07/writing-a-clap-synthesizer-in-rust-part-1/*
  - *https://kwarf.com/2024/07/writing-a-clap-synthesizer-in-rust-part-2/*
- *https://github.com/PaulBatchelor/sndkit/blob/master/dsp/dcblocker.org:* DC blocking filter explanation and implementation in C.
  - *https://ccrma.stanford.edu/~jos/filters/DC_Blocker.html*
- *https://cs.wellesley.edu/~cs203/lecture_materials/freq_phase_modulation/freq_phase_modulation.pdf:* Lecture slides on frequency and phase modulation.

[LV2 plugins]: https://lv2plug.in/
[clack]: https://github.com/prokopyl/clack
