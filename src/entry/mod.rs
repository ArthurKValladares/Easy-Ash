use crate::ApplicationInfo;
use anyhow::Result;
use ash::vk;

pub struct Entry {
    entry: ash::Entry,
    instance: ash::Instance,
}

impl Entry {
    pub fn new(application_info: ApplicationInfo) -> Result<Self> {
        let entry = ash::Entry::linked();
        let application_info: vk::ApplicationInfo = application_info.into();

        let create_info = vk::InstanceCreateInfo::builder().application_info(&application_info);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        Ok(Self { entry, instance })
    }
}
