mod debug;

use crate::ApplicationInfo;
use anyhow::Result;
use ash::vk;
use debug::DebugUtils;
use raw_window_handle::HasRawWindowHandle;
use std::ffi::{CStr, CString};

pub struct InstanceInfo {
    debug_layers: bool,
}

impl Default for InstanceInfo {
    fn default() -> Self {
        Self { debug_layers: true }
    }
}

impl InstanceInfo {
    pub fn with_debug_layers(mut self, debug_layers: bool) -> Self {
        self.debug_layers = debug_layers;
        self
    }

    fn layer_names(&self) -> Vec<CString> {
        let mut layers = Vec::new();
        if self.debug_layers {
            layers.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());
        }
        layers
    }

    fn extensions(&self) -> Vec<&'static CStr> {
        let mut extensions = Vec::new();
        if self.debug_layers {
            extensions.push(ash::extensions::ext::DebugUtils::name())
        }
        extensions
    }
}

pub struct Entry {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    debug_utils: Option<DebugUtils>,
}

impl Entry {
    pub fn new(
        application_info: ApplicationInfo,
        instance_info: InstanceInfo,
        window_handle: &dyn HasRawWindowHandle,
    ) -> Result<Self> {
        let entry = ash::Entry::linked();
        let application_info: vk::ApplicationInfo = application_info.into();

        let layer_names = instance_info.layer_names();
        let extensions = instance_info.extensions();

        let layer_names_raw = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect::<Vec<*const std::os::raw::c_char>>();

        let extensions_raw = extensions
            .iter()
            .map(|extension| extension.as_ptr())
            .chain(ash_window::enumerate_required_extensions(window_handle)?.to_vec())
            .collect::<Vec<*const std::os::raw::c_char>>();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extensions_raw);

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        let debug_utils = if instance_info.debug_layers {
            Some(DebugUtils::new(&entry, &instance)?)
        } else {
            None
        };

        Ok(Self {
            entry,
            instance,
            debug_utils,
        })
    }

    pub unsafe fn clean(&self) {
        if let Some(debug_utils) = &self.debug_utils {
            debug_utils.clean();
        }
        self.instance.destroy_instance(None);
    }
}
