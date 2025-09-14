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

# Installation

I publish binaries for **windows** and **linux** on the releases section on github. Download and place fox3osc in your clap plugin folder. If your CPU isn't **x86_64**, or you're on neither OS download the source code and build it from source.

## Installation from source

To build fox3osc, you will need **[cargo]**.

Build the plugin with cargo (`cargo build --release`) and copy the built binary from `./target/release/` to your clap plugin folder. Make sure to rename the extension to `.clap`. On MacOS, you'll need to make an app bundle from scratch instead.

Alternatively, a **[justfile]** is provided to make installation super convenient. Just can be installed with cargo very easily with `cargo install just`. `just install` will then install the plugin to your user clap plugin folder (eg: `~/.clap` on linux).

Building the plugin with the justfile (Running `just`) uses the nightly version of rustc and cargo in order to agressively optimize the size of the final binary. It will download and install the latest version of the nightly compiler and the `rust-src` rustup component for nightly, if not present. The latter is needed for the `-Z build-std` feature on cargo to work.

Of course the nightly compiler isn't needed to build fox3osc, this is just so the final binary can be as small as possible.

**This is how I reccomend you build fox3osc:**
```shell
cargo install just  # In case you don't have just already installed
just                # Or alternatively "cargo build --release" if you don't care about having a smaller binary
just install
```

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
