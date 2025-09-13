#![allow(clippy::upper_case_acronyms, clippy::type_complexity)]
#![deny(clippy::undocumented_unsafe_blocks)]

use clack_extensions::{
    audio_ports::PluginAudioPorts, note_ports::PluginNotePorts, params::PluginParams,
    state::PluginState,
};
use clack_plugin::prelude::*;

use crate::{
    audio_processor::Fox3oscAudioProcessor, main_thread::Fox3oscMainThread, shared::Fox3oscShared,
};

mod audio_processor;
mod consts;
mod key;
mod main_thread;
mod shared;

struct Fox3oscDescriptor {
    author: &'static str,
    id: String,
}

impl Fox3oscDescriptor {
    const NAME: &str = env!("CARGO_PKG_NAME");

    pub fn new() -> Self {
        let author = env!("CARGO_PKG_AUTHORS")
            .split(':')
            .next()
            .unwrap_or_default();

        let id = format!("com.{author}.{}", Self::NAME);
        Self { author, id }
    }

    pub const fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub const fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }

    pub const fn url(&self) -> &'static str {
        env!("CARGO_PKG_HOMEPAGE")
    }

    pub const fn name(&self) -> &'static str {
        Self::NAME
    }

    pub fn author(&self) -> &'static str {
        self.author
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

struct Fox3osc {}

impl Plugin for Fox3osc {
    type AudioProcessor<'a> = Fox3oscAudioProcessor<'a>;
    type Shared<'a> = Fox3oscShared;
    type MainThread<'a> = Fox3oscMainThread<'a>;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&Self::Shared<'_>>,
    ) {
        builder
            .register::<PluginAudioPorts>()
            .register::<PluginNotePorts>()
            .register::<PluginParams>()
            .register::<PluginState>();
    }
}

impl DefaultPluginFactory for Fox3osc {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        let descriptor = Fox3oscDescriptor::new();

        PluginDescriptor::new(descriptor.id(), descriptor.name())
            .with_vendor(descriptor.author())
            .with_version(descriptor.version())
            .with_description(descriptor.description())
            .with_url(descriptor.url())
            .with_features([INSTRUMENT, SYNTHESIZER, MONO])
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(Fox3oscShared::default())
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(Fox3oscMainThread::new(shared))
    }
}

clack_export_entry!(SinglePluginEntry<Fox3osc>);
