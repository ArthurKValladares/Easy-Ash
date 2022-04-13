mod api_version;

use self::api_version::ApiVersion;
use easy_versions::VersionSingle;

#[derive(Debug)]
pub struct ApplicationInfo<'a> {
    application_name: &'a str,
    application_version: VersionSingle<u32>,
    engine_name: &'a str,
    engine_version: VersionSingle<u32>,
    api_version: ApiVersion,
}

impl<'a> Default for ApplicationInfo<'a> {
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

impl<'a> ApplicationInfo<'a> {
    pub fn with_application_name(mut self, application_name: &'a str) -> Self {
        self.application_name = application_name;
        self
    }

    pub fn with_application_version(mut self, application_version: VersionSingle<u32>) -> Self {
        self.application_version = application_version;
        self
    }

    pub fn with_engine_name(mut self, engine_name: &'a str) -> Self {
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
