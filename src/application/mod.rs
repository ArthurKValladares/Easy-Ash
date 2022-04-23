mod api_version;

pub use self::api_version::ApiVersion;
use ash::vk;
use easy_versions::VersionSingle;

#[derive(Debug)]
pub struct ApplicationInfo {
    application_name: &'static str,
    application_version: VersionSingle<u32>,
    engine_name: &'static str,
    engine_version: VersionSingle<u32>,
    api_version: ApiVersion,
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        Self {
            application_name: "Easy Ash Application",
            application_version: VersionSingle::<u32>::new(1),
            engine_name: "Easy Ash Engine",
            engine_version: VersionSingle::<u32>::new(1),
            api_version: ApiVersion::new(0, 1, 0, 0),
        }
    }
}

impl ApplicationInfo {
    pub fn with_application_name(mut self, application_name: &'static str) -> Self {
        self.application_name = application_name;
        self
    }

    pub fn with_application_version(mut self, application_version: VersionSingle<u32>) -> Self {
        self.application_version = application_version;
        self
    }

    pub fn with_engine_name(mut self, engine_name: &'static str) -> Self {
        self.engine_name = engine_name;
        self
    }

    pub fn with_engine_version(mut self, engine_version: VersionSingle<u32>) -> Self {
        self.engine_version = engine_version;
        self
    }

    pub fn with_api_version(mut self, api_version: ApiVersion) -> Self {
        self.api_version = api_version;
        self
    }
}

impl From<ApplicationInfo> for vk::ApplicationInfo {
    fn from(app_info: ApplicationInfo) -> vk::ApplicationInfo {
        let (variant, major, minor, patch) = app_info.api_version.as_parts();
        vk::ApplicationInfo {
            p_application_name: app_info.application_name.as_ptr() as *const _,
            application_version: app_info.application_version.version(),
            p_engine_name: app_info.engine_name.as_ptr() as *const _,
            engine_version: app_info.engine_version.version(),
            api_version: vk::make_api_version(variant, major, minor, patch),
            ..Default::default()
        }
    }
}
