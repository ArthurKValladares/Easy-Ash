use easy_versions::VersionTriple;

#[derive(Debug, Copy, Clone)]
pub struct ApiVersion {
    triple: VersionTriple<u32>,
    variant: u32,
}

impl ApiVersion {
    pub fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self {
            triple: VersionTriple::new(major, minor, patch),
            variant,
        }
    }

    pub fn variant(&self) -> u32 {
        self.variant
    }

    pub fn major(&self) -> u32 {
        self.triple.major()
    }

    pub fn minor(&self) -> u32 {
        self.triple.minor()
    }

    pub fn patch(&self) -> u32 {
        self.triple.patch()
    }

    pub fn as_parts(&self) -> (u32, u32, u32, u32) {
        (
            self.variant,
            self.triple.major(),
            self.triple.minor(),
            self.triple.patch(),
        )
    }
}

impl From<VersionTriple<u32>> for ApiVersion {
    fn from(triple: VersionTriple<u32>) -> ApiVersion {
        ApiVersion::new(0, triple.major(), triple.minor(), triple.patch())
    }
}
