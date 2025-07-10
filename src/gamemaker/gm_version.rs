use std::fmt::Formatter;
use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::element::GMElement;
use crate::gamemaker::serialize::DataBuilder;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LTSBranch {
    Pre2022_0,
    LTS2022_0,
    Post2022_0,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVersion {
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub build: u32,
    pub branch: LTSBranch,
}

impl GMVersion {
    pub fn new(major: u32, minor: u32, release: u32, build: u32, branch: LTSBranch) -> Self {
        Self { major, minor, release, build, branch }
    }
}

impl std::fmt::Display for GMVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let branch_str = match self.branch {
            LTSBranch::Pre2022_0 => "PreLTS",
            LTSBranch::LTS2022_0 => "LTS",
            LTSBranch::Post2022_0 => "PostLTS",
        };
        write!(f, "{}.{}.{}.{} ({branch_str})", self.major, self.minor, self.release, self.build)
    }
}

impl GMVersion {
    pub fn is_version_at_least<V: Into<GMVersionReq>>(&self, version_req: V) -> bool {
        let ver: GMVersionReq = version_req.into();
        if ver.non_lts && self.branch < LTSBranch::Post2022_0 {
            return false
        }
        if self.major != ver.major {
            return self.major > ver.major;
        }
        if self.minor != ver.minor {
            return self.minor > ver.minor;
        }
        if self.release != ver.release {
            return self.release > ver.release;
        }
        if self.build != ver.build {
            return self.build > ver.build;
        }
        true   // The version is exactly what was supplied.
    }

    pub fn set_version_at_least<V: Into<GMVersionReq>>(&mut self, version_req: V) -> Result<(), String> {
        let new_ver: GMVersionReq = version_req.into();
        if !matches!(new_ver.major, 2|2022|2023|2024) {
            return Err(format!(
                "Tried to set GameMaker Version to {} which is not allowed for original GameMaker Version {}",
                new_ver, self,
            ))
        }
        if self.is_version_at_least(new_ver.clone()) {
            return Ok(())   // only override version if new version is higher
        }
        self.major = new_ver.major;
        self.minor = new_ver.minor;
        self.release = new_ver.release;
        self.build = new_ver.build;
        if new_ver.non_lts {
            self.branch = LTSBranch::Post2022_0;
        }
        Ok(())
    }
}

impl GMElement for GMVersion {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let major: u32 = reader.read_u32()?;
        let minor: u32 = reader.read_u32()?;
        let release: u32 = reader.read_u32()?;
        let build: u32 = reader.read_u32()?;
        // since gen8 gm version is stuck on maximum 2.0.0.0; LTS will (initially) always be Pre2022_0
        Ok(GMVersion::new(major, minor, release, build, LTSBranch::Pre2022_0))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.major);
        builder.write_u32(self.minor);
        builder.write_u32(self.release);
        builder.write_u32(self.build);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMVersionReq {
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub build: u32,
    pub non_lts: bool,
}

impl GMVersionReq {
    pub fn none() -> Self {
        Self {
            major: 0,
            minor: 0,
            release: 0,
            build: 0,
            non_lts: false,
        }
    }
}

impl From<(u32, u32)> for GMVersionReq {
    fn from((major, minor): (u32, u32)) -> Self {
        Self {
            major,
            minor,
            release: 0,
            build: 0,
            non_lts: false,
        }
    }
}

impl From<(u32, u32, u32)> for GMVersionReq {
    fn from((major, minor, release): (u32, u32, u32)) -> Self {
        Self {
            major,
            minor,
            release,
            build: 0,
            non_lts: false,
        }
    }
}

impl From<(u32, u32, u32, u32)> for GMVersionReq {
    fn from((major, minor, release, build): (u32, u32, u32, u32)) -> Self {
        Self {
            major,
            minor,
            release,
            build,
            non_lts: false,
        }
    }
}

impl From<(u32, u32, LTSBranch)> for GMVersionReq {
    fn from((major, minor, lts): (u32, u32, LTSBranch)) -> Self {
        Self {
            major,
            minor,
            release: 0,
            build: 0,
            non_lts: matches!(lts, LTSBranch::Post2022_0),
        }
    }
}

impl From<(u32, u32, u32, LTSBranch)> for GMVersionReq {
    fn from((major, minor, release, lts): (u32, u32, u32, LTSBranch)) -> Self {
        Self {
            major,
            minor,
            release,
            build: 0,
            non_lts: matches!(lts, LTSBranch::Post2022_0),
        }
    }
}

impl From<(u32, u32, u32, u32, LTSBranch)> for GMVersionReq {
    fn from((major, minor, release, build, lts): (u32, u32, u32, u32, LTSBranch)) -> Self {
        Self {
            major,
            minor,
            release,
            build,
            non_lts: matches!(lts, LTSBranch::Post2022_0),
        }
    }
}

impl std::fmt::Display for GMVersionReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lts_str = if self.non_lts { " (Non LTS)" } else { "" };
        write!(f, "{}.{}.{}.{}{lts_str}", self.major, self.minor, self.release, self.build)
    }
}


