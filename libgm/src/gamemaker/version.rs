//! Contains GameMaker IDE Version types and abstractions to check and set versions.

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
};

/// Different GameMaker release branches. LTS has some but not all features of equivalent newer versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum LTSBranch {
    /// Before LTS was even introduced (`major < 2022`).
    PreLTS,

    /// Long-Term Support branch.
    /// YoYoGames updates minor bugfixes here, but doesn't make breaking changes
    /// (except in 2023.6?).
    LTS,

    /// New Version but not the Long-Term Support branch.
    /// YoYo Games introduces all new features here, some of which may break your project.
    PostLTS,
}

impl Display for LTSBranch {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let string = match self {
            Self::PreLTS => "PreLTS",
            Self::LTS => "LTS",
            Self::PostLTS => "PostLTS",
        };
        f.write_str(string)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
/// A GameMaker Studio Version.
///
/// Theoretically, this is only the version of the IDE this game was made in.
/// However, this version struct is also used for file format purposes in this library.
/// This is because it is more accurate than the `WAD Version` in `GMGeneralInfo`,
/// which is no longer updated (stuck since WAD 17).
///
/// This version struct is also not updated by YoYo Games since GM:S 2 and its
/// raw `GEN8` version is stuck on `2.0.0.0`.
/// This library uses version detection to detect the approximate GameMaker version
/// so that the file format can be deserialized properly.
pub struct GMVersion {
    /// If greater than 1, serialization produces "2.0.0.0" due to the flag no longer updating in `data.win`.
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub build: u32,
    /// Different GameMaker release branches. LTS has some but not all features of equivalent newer versions.
    pub branch: LTSBranch,
}

impl GMVersion {
    #[must_use]
    pub const fn new(major: u32, minor: u32, release: u32, build: u32, branch: LTSBranch) -> Self {
        Self { major, minor, release, build, branch }
    }

    /// Creates a temporary placeholder [`GMVersion`] to avoid [`Option`]s.
    /// Make sure to never read this value before properly initialized (overwritten)!
    #[must_use]
    pub(crate) const fn stub() -> Self {
        Self::new(1337, 1337, 1337, 1337, LTSBranch::PreLTS)
    }
}

impl Display for GMVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_version(f, self.major, self.minor, self.release, self.build)?;
        if self.major >= 2022 {
            write!(f, " ({})", self.branch)?;
        }
        Ok(())
    }
}

impl GMVersion {
    // TODO(const): Make these functions `const` when Into, PartialEq and PartialOrd are const-stable.

    /// Checks if the current version is at least the specified version.
    ///
    /// Compares major, minor, release, and build numbers in sequence,
    /// and also considers the branch for non-LTS versions.
    ///
    /// # Parameters
    /// - `version_req`: The version requirement to compare against (convertible into [`GMVersionReq`]).
    ///
    /// # Returns
    /// `true` if `self` is greater than or equal to `version_req`.
    ///
    /// # Notes
    /// You can also compare `GMVersion` against other `GMVersion`s or `GMVersionReq`s directly.
    /// Example:
    /// ```ignore
    /// let version_requirement: GMVersionReq = (2023, 6).into();
    /// if gm_data.general_info.version < version_requirement {
    ///     // ...
    /// }
    /// ```
    /// This is mostly useful for non-constant/dynamic [`GMVersionReq`]s.
    #[must_use]
    pub fn is_version_at_least(&self, version_req: impl Into<GMVersionReq>) -> bool {
        self >= &version_req.into()
    }

    pub fn set_version(&mut self, req: impl Into<GMVersionReq>) {
        let new = req.into();
        self.major = new.major;
        self.minor = new.minor;
        self.release = new.release;
        self.build = new.build;
        if new.post_lts {
            self.branch = LTSBranch::PostLTS;
        } else if new.major >= 2022 {
            // TODO: is this correct?
            self.branch = LTSBranch::LTS;
        }
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
    pub fn set_version_at_least(&mut self, req: impl Into<GMVersionReq>) -> Result<()> {
        let new_ver: GMVersionReq = req.into();
        if !matches!(new_ver.major, 2 | 2022..=2026) {
            let comment = if new_ver.major > 2026 && new_ver.major < 2100 {
                format!(
                    "! If the current year is {} or greater, please contact the \
                    maintainer of this project to update the version validation.",
                    new_ver.major
                )
            } else {
                String::new()
            };
            bail!("Upgrading GameMaker Version from {self} to {new_ver} is not allowed{comment}");
        }

        if *self < new_ver {
            self.set_version(new_ver);
        }
        Ok(())
    }
}

impl PartialEq<GMVersionReq> for GMVersion {
    fn eq(&self, req: &GMVersionReq) -> bool {
        if req.post_lts {
            return self.branch == LTSBranch::PostLTS;
        }

        self.major == req.major
            && self.minor == req.minor
            && self.release == req.release
            && self.build == req.build
    }
}

impl PartialOrd<GMVersionReq> for GMVersion {
    fn partial_cmp(&self, req: &GMVersionReq) -> Option<Ordering> {
        if req.post_lts && self.branch < LTSBranch::PostLTS {
            return Some(Ordering::Less);
        }

        macro_rules! cmp {
            ($part:ident) => {
                match self.$part.cmp(&req.$part) {
                    Ordering::Equal => {},
                    other => return Some(other),
                }
            };
        }

        cmp!(major);
        cmp!(minor);
        cmp!(release);
        cmp!(build);

        Some(Ordering::Equal)
    }
}

impl GMElement for GMVersion {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let major = reader.read_u32()?;
        let minor = reader.read_u32()?;
        let release = reader.read_u32()?;
        let build = reader.read_u32()?;
        // Since the GEN8 Version is stuck on maximum 2.0.0.0; LTS will (initially) always be PreLTS
        Ok(Self::new(major, minor, release, build, LTSBranch::PreLTS))
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.major);
        builder.write_u32(self.minor);
        builder.write_u32(self.release);
        builder.write_u32(self.build);
        Ok(())
    }
}

/// A GameMaker Version Requirement for checking if the game's version is equal to or higher than x.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMVersionReq {
    pub major: u32,
    pub minor: u32,
    pub release: u32,
    pub build: u32,
    /// Only makes sense for `major >= 2022` since LTS didn't exist before.
    /// If [true], the version's branch has to be [`LTSBranch::PostLTS`].
    pub post_lts: bool,
}

impl GMVersionReq {
    #[must_use]
    pub const fn new(major: u32, minor: u32, release: u32, build: u32, lts: LTSBranch) -> Self {
        let post_lts: bool = matches!(lts, LTSBranch::PostLTS);
        Self { major, minor, release, build, post_lts }
    }

    pub const NONE: Self = Self::new(0, 0, 0, 0, LTSBranch::PreLTS);
}

impl From<(u32, u32)> for GMVersionReq {
    fn from((major, minor): (u32, u32)) -> Self {
        Self {
            major,
            minor,
            release: 0,
            build: 0,
            post_lts: false,
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
            post_lts: false,
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
            post_lts: false,
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
            post_lts: matches!(lts, LTSBranch::PostLTS),
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
            post_lts: matches!(lts, LTSBranch::PostLTS),
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
            post_lts: matches!(lts, LTSBranch::PostLTS),
        }
    }
}

impl Display for GMVersionReq {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write_version(f, self.major, self.minor, self.release, self.build)?;
        if self.post_lts {
            write!(f, " (Post LTS)")?;
        }
        Ok(())
    }
}

fn write_version(
    f: &mut Formatter,
    major: u32,
    minor: u32,
    release: u32,
    build: u32,
) -> std::fmt::Result {
    write!(f, "{major}")?;
    match (minor, release, build) {
        (0, 0, 0) => Ok(()),
        (minor, 0, 0) => write!(f, ".{minor}"),
        (minor, release, 0) => write!(f, ".{minor}.{release}"),
        (minor, release, build) => {
            write!(f, ".{minor}.{release}.{build}")
        },
    }
}
