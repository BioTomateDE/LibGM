use crate::prelude::*;
use std::fmt::Formatter;
use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::elements::GMElement;
use crate::gamemaker::serialize::DataBuilder;


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LTSBranch {
    PreLTS,
    LTS,
    PostLTS,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVersion {
    /// If greater than 1, serialization produces "2.0.0.0" due to the flag no longer updating in `data.win`
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub build: u32,
    /// Different GameMaker release branches. LTS has some but not all features of equivalent newer versions.
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
            LTSBranch::PreLTS => "PreLTS",
            LTSBranch::LTS => "LTS",
            LTSBranch::PostLTS => "PostLTS",
        };
        write!(f, "{}.{}.{}.{} ({branch_str})", self.major, self.minor, self.release, self.build)
    }
}

impl GMVersion {
    /// Checks if the current version is at least the specified version.
    ///
    /// Compares major, minor, release, and build numbers in sequence,
    /// and also considers the branch for non-LTS versions.
    ///
    /// # Parameters
    /// - `version_req`: The version requirement to compare against (convertible into `GMVersionReq`).
    ///
    /// # Returns
    /// `true` if `self` is greater than or equal to `version_req`.
    pub fn is_version_at_least<V: Into<GMVersionReq>>(&self, version_req: V) -> bool {
        let ver: GMVersionReq = version_req.into();
        if ver.non_lts && self.branch < LTSBranch::PostLTS {
            return false
        }
        if self.major != ver.major {
            return self.major > ver.major
        }
        if self.minor != ver.minor {
            return self.minor > ver.minor
        }
        if self.release != ver.release {
            return self.release > ver.release
        }
        if self.build != ver.build {
            return self.build > ver.build
        }
        true   // The version is exactly what was supplied.
    }

    /// Sets the version to at least the specified version.
    /// Only updates if the new version is higher than the current one.
    ///
    /// # Parameters
    /// - `version_req`: The minimum version to set (convertible into `GMVersionReq`).
    ///
    /// # Errors
    /// Returns an error if the requested version is not allowed (invalid major version).
    ///
    /// # Notes
    /// Setting a non-LTS version updates the branch accordingly.
    pub fn set_version_at_least<V: Into<GMVersionReq>>(&mut self, version_req: V) -> Result<()> {
        let new_ver: GMVersionReq = version_req.into();
        if !matches!(new_ver.major, 2|2022|2023|2024|2025) {
            let comment = if new_ver.major > 2025 && new_ver.major < 2100 {
                format!("! If the current year is {} or greater, please contact the maintainer of this project to update the version validation.", new_ver.major)
            } else {
                String::new()
            };
            bail!("Upgrading GameMaker Version from {self} to {new_ver} is not allowed{comment}");
        }

        if self.is_version_at_least(new_ver.clone()) {
            return Ok(())   // only override version if new version is higher
        }
        self.major = new_ver.major;
        self.minor = new_ver.minor;
        self.release = new_ver.release;
        self.build = new_ver.build;
        if new_ver.non_lts {
            self.branch = LTSBranch::PostLTS;
        }
        Ok(())
    }
}


impl GMElement for GMVersion {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let major = reader.read_u32()?;
        let minor = reader.read_u32()?;
        let release = reader.read_u32()?;
        let build = reader.read_u32()?;
        // Since the GEN8 Version is stuck on maximum 2.0.0.0; LTS will (initially) always be PreLTS
        Ok(GMVersion::new(major, minor, release, build, LTSBranch::PreLTS))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
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
            non_lts: matches!(lts, LTSBranch::PostLTS),
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
            non_lts: matches!(lts, LTSBranch::PostLTS),
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
            non_lts: matches!(lts, LTSBranch::PostLTS),
        }
    }
}

impl std::fmt::Display for GMVersionReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lts_str = if self.non_lts { " (Non LTS)" } else { "" };
        write!(f, "{}.{}.{}.{}{lts_str}", self.major, self.minor, self.release, self.build)
    }
}


