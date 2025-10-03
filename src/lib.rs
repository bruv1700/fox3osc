#![allow(clippy::upper_case_acronyms, clippy::type_complexity)]
#![deny(clippy::undocumented_unsafe_blocks)]

use std::ffi::CStr;

use clack_extensions::{
    audio_ports::PluginAudioPorts, note_ports::PluginNotePorts, params::PluginParams,
    state::PluginState,
};
use clack_plugin::entry::prelude::*;
use clack_plugin::prelude::*;

use crate::{
    audio_processor::Fox3oscAudioProcessor,
    consts::{AUTHOR, PLUGIN_COUNT},
    main_thread::Fox3oscMainThread,
    shared::Fox3oscShared,
};

mod audio_processor;
mod consts;
mod key;
mod main_thread;
mod math;
mod shared;

struct Fox3oscDescriptor {
    name: &'static str,
    id: &'static str,
}

macro_rules! fox3osc_descriptor {
    ($name:expr) => {
        const {
            const NAME: &'static str = $name;
            const NAME_URI_VALID1: &'static str = const_str::replace!(NAME, "(", "_");
            const NAME_URI_VALID2: &'static str = const_str::replace!(NAME_URI_VALID1, ")", "_");
            const NAME_URI_VALID3: &'static str = const_str::replace!(NAME_URI_VALID2, " ", "_");

            Fox3oscDescriptor {
                name: NAME,
                id: const_str::concat!("com.", AUTHOR, ".", NAME_URI_VALID3),
            }
        }
    };
}

impl Fox3oscDescriptor {
    pub const fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub const fn url(&self) -> &'static str {
        env!("CARGO_PKG_HOMEPAGE")
    }

    pub const fn description(&self) -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }

    pub const fn author(&self) -> &'static str {
        AUTHOR
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn id(&self) -> &'static str {
        self.id
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

struct Fox3oscEntry {
    plugin_factory: PluginFactoryWrapper<Fox3oscFactory>,
}

impl Entry for Fox3oscEntry {
    fn new(_bundle_path: &CStr) -> Result<Self, EntryLoadError> {
        Ok(Self {
            plugin_factory: PluginFactoryWrapper::new(Fox3oscFactory::new()),
        })
    }

    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>) {
        builder.register_factory(&self.plugin_factory);
    }
}

static PLUGIN_DESCRIPTORS: [Fox3oscDescriptor; PLUGIN_COUNT] = [
    fox3osc_descriptor!("fox3osc"),
    #[cfg(feature = "15tet")]
    fox3osc_descriptor!("fox3osc (15-tet)"),
    #[cfg(feature = "17tet")]
    fox3osc_descriptor!("fox3osc (17-tet)"),
    #[cfg(feature = "19tet")]
    fox3osc_descriptor!("fox3osc (19-tet)"),
    #[cfg(feature = "22tet")]
    fox3osc_descriptor!("fox3osc (22-tet)"),
    #[cfg(feature = "23tet")]
    fox3osc_descriptor!("fox3osc (23-tet)"),
    #[cfg(feature = "24tet")]
    fox3osc_descriptor!("fox3osc (24-tet)"),
];

static PLUGIN_TEMPERAMENTS: [f32; PLUGIN_COUNT] = [
    12.0,
    #[cfg(feature = "15tet")]
    15.0,
    #[cfg(feature = "17tet")]
    17.0,
    #[cfg(feature = "19tet")]
    19.0,
    #[cfg(feature = "22tet")]
    22.0,
    #[cfg(feature = "23tet")]
    23.0,
    #[cfg(feature = "24tet")]
    24.0,
];

struct Fox3oscFactory {
    plugin_descriptors: [PluginDescriptor; PLUGIN_COUNT],
}

impl Fox3oscFactory {
    pub fn new() -> Self {
        use clack_plugin::plugin::features::*;

        let plugin_descriptors = std::array::from_fn(|i| {
            let descriptor = &PLUGIN_DESCRIPTORS[i];
            PluginDescriptor::new(descriptor.id(), descriptor.name())
                .with_vendor(descriptor.author())
                .with_version(descriptor.version())
                .with_description(descriptor.description())
                .with_url(descriptor.url())
                .with_features([INSTRUMENT, SYNTHESIZER, MONO])
        });

        Self { plugin_descriptors }
    }
}

impl PluginFactory for Fox3oscFactory {
    fn plugin_count(&self) -> u32 {
        const { PLUGIN_COUNT as u32 }
    }

    fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptor> {
        self.plugin_descriptors.get(index as usize)
    }

    fn create_plugin<'a>(
        &'a self,
        host_info: HostInfo<'a>,
        plugin_id: &CStr,
    ) -> Option<PluginInstance<'a>> {
        self.plugin_descriptors
            .iter()
            .zip(PLUGIN_TEMPERAMENTS)
            .find_map(|(plugin_descriptor, plugin_temperament)| {
                if plugin_id == plugin_descriptor.id() {
                    let instance = PluginInstance::new::<Fox3osc>(
                        host_info,
                        plugin_descriptor,
                        move |_host| Ok(Fox3oscShared::new(plugin_temperament)),
                        |_host, shared| Ok(Fox3oscMainThread::new(shared)),
                    );

                    Some(instance)
                } else {
                    None
                }
            })
    }
}

clack_export_entry!(Fox3oscEntry);
