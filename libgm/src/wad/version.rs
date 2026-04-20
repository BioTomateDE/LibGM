//! Contains GameMaker IDE Version types and abstractions to check and set
//! versions.

use core::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;

/// Different GameMaker release branches. LTS has some but not all features of
/// equivalent newer versions.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum LTSBranch {
    /// Before LTS was even introduced (`major < 2022`).
    PreLTS,

    /// Long-Term Support branch.
    /// YoYoGames updates minor bugfixes here, but doesn't make breaking changes
    /// (except in 2023.6?).
    LTS,

    /// New Version but not the Long-Term Support branch.
    /// YoYo Games introduces all new features here, some of which may break
    /// your project.
    PostLTS,
}

/// Custom implementation because `PreLTS`
/// and `LTS` cannot be properly compared.
impl PartialOrd for LTSBranch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use LTSBranch::*;
        use Ordering::*;

        match (self, other) {
            (PreLTS, PreLTS) => Some(Equal),
            (PreLTS, LTS) => None,
            (PreLTS, PostLTS) => Some(Less),
            (LTS, PreLTS) => None,
            (LTS, LTS) => Some(Equal),
            (LTS, PostLTS) => Some(Less),
            (PostLTS, PreLTS) => Some(Greater),
            (PostLTS, LTS) => Some(Less),
            (PostLTS, PostLTS) => Some(Equal),
        }
    }
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

/// A GameMaker Studio Version.
///
/// Theoretically, this is only the version of the IDE this game was made in.
/// However, this version struct is also used for file format purposes in this
/// library. This is because it is more accurate than the `WAD Version` in
/// `GMGeneralInfo`, which is no longer updated (stuck since WAD 17).
///
/// This version struct is also not updated by YoYo Games since GM:S 2 and its
/// raw `GEN8` version is stuck on `2.0.0.0`.
/// This library uses version detection to detect the approximate GameMaker
/// version so that the file format can be deserialized properly.
#[derive(Debug, Clone, Copy)]
pub struct GMVersion {
    /// The most significant version part.
    /// This can be 1, 2, or a year after 2021.
    ///
    /// If greater than 1, serialization produces "2.0.0.0" due to the flag no
    /// longer updating in `data.win`.
    pub major: u32,

    /// The second-most significant version part.
    pub minor: u32,

    /// The third-most significant version part.
    pub release: u32,

    /// The fourth-most (least) significant version part.
    pub build: u32,

    /// Different GameMaker release branches.
    /// LTS has some but not all features of equivalent newer versions.
    /// See [`LTSBranch`] for more information.
    pub branch: LTSBranch,
}

impl GMVersion {
    // TODO(const): Make these functions `const` when Into, PartialEq and PartialOrd
    // are const-stable.

    /// The GameMaker Version 2.0.0.0 (Pre LTS).
    pub const GMS2: Self = Self::new(2, 0, 0, 0, LTSBranch::PreLTS);
    /// The pseudo-version "infinity" (highest possible values).
    ///
    /// This is only useful for comparing against other `GMVersion`s
    /// dynamically.
    pub const INF: Self = Self::new(u32::MAX, u32::MAX, u32::MAX, u32::MAX, LTSBranch::PostLTS);
    /// The pseudo-version 0.0.0.0 (Pre LTS).
    ///
    /// This is only useful for comparing against other `GMVersion`s
    /// dynamically.
    pub const NULL: Self = Self::new(0, 0, 0, 0, LTSBranch::PreLTS);

    /// Creates a new [`GMVersion`] with the given version parts and branch.
    #[must_use]
    pub const fn new(major: u32, minor: u32, release: u32, build: u32, branch: LTSBranch) -> Self {
        Self { major, minor, release, build, branch }
    }

    /// Sets this [`GMVersion`] to the specified [`GMVersion`].
    ///
    /// The LTS branch is only updated if the new branch
    /// is greater than the current branch.
    pub fn set_version(&mut self, new_version: impl ToGMVersion) {
        let new: Self = new_version.into_gm_version();
        self.major = new.major;
        self.minor = new.minor;
        self.release = new.release;
        self.build = new.build;
        if new.branch > self.branch {
            self.branch = new.branch;
        }
    }
}

impl GMElement for GMVersion {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let major = reader.read_u32()?;
        let minor = reader.read_u32()?;
        let release = reader.read_u32()?;
        let build = reader.read_u32()?;
        // Since the GEN8 Version is stuck on maximum 2.0.0.0; LTS will (initially)
        // always be PreLTS
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

impl Default for GMVersion {
    fn default() -> Self {
        Self::GMS2
    }
}

impl Display for GMVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let major = self.major;
        let minor = self.minor;
        let release = self.release;
        let build = self.build;
        write!(f, "{major}.{minor}")?;
        match (release, build) {
            (0, 0) => {}
            (_, 0) => write!(f, ".{release}")?,
            (_, _) => write!(f, ".{build}")?,
        }
        if major >= 2022 {
            write!(f, " ({})", self.branch)?;
        }
        Ok(())
    }
}

impl<T: ToGMVersion> PartialEq<T> for GMVersion {
    fn eq(&self, other: &T) -> bool {
        let a = self;
        let b = other.to_gm_version();
        a.major == b.major
            && a.minor == b.minor
            && a.release == b.release
            && a.build == b.build
            && a.branch == b.branch
    }
}

impl<T: ToGMVersion> PartialOrd<T> for GMVersion {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        let other = other.to_gm_version();
        compare(self, &other)
    }
}

fn compare(a: &GMVersion, b: &GMVersion) -> Option<Ordering> {
    match a.branch.partial_cmp(&b.branch) {
        Some(Ordering::Equal) | None => {}
        ord => return ord,
    }
    match a.major.partial_cmp(&b.major) {
        Some(Ordering::Equal) => {}
        ord => return ord,
    }
    match a.minor.partial_cmp(&b.minor) {
        Some(Ordering::Equal) => {}
        ord => return ord,
    }
    match a.release.partial_cmp(&b.release) {
        Some(Ordering::Equal) => {}
        ord => return ord,
    }
    a.build.partial_cmp(&b.build)
}

pub trait ToGMVersion: Copy {
    #[must_use]
    fn to_gm_version(&self) -> GMVersion;

    #[must_use]
    fn into_gm_version(self) -> GMVersion {
        self.to_gm_version()
    }
}

impl ToGMVersion for GMVersion {
    fn to_gm_version(&self) -> GMVersion {
        *self
    }

    fn into_gm_version(self) -> GMVersion {
        self
    }
}

impl ToGMVersion for &GMVersion {
    fn to_gm_version(&self) -> GMVersion {
        **self
    }
}

impl ToGMVersion for (u32, u32, u32, u32, LTSBranch) {
    fn to_gm_version(&self) -> GMVersion {
        GMVersion::new(self.0, self.1, self.2, self.3, self.4)
    }
}

impl ToGMVersion for (u32, u32, u32, LTSBranch) {
    fn to_gm_version(&self) -> GMVersion {
        GMVersion::new(self.0, self.1, self.2, 0, self.3)
    }
}

impl ToGMVersion for (u32, u32, LTSBranch) {
    fn to_gm_version(&self) -> GMVersion {
        GMVersion::new(self.0, self.1, 0, 0, self.2)
    }
}

impl ToGMVersion for (u32, u32, u32, u32) {
    fn to_gm_version(&self) -> GMVersion {
        GMVersion::new(self.0, self.1, self.2, self.3, LTSBranch::PreLTS)
    }
}

impl ToGMVersion for (u32, u32, u32) {
    fn to_gm_version(&self) -> GMVersion {
        GMVersion::new(self.0, self.1, self.2, 0, LTSBranch::PreLTS)
    }
}

impl ToGMVersion for (u32, u32) {
    fn to_gm_version(&self) -> GMVersion {
        GMVersion::new(self.0, self.1, 0, 0, LTSBranch::PreLTS)
    }
}

impl ToGMVersion for u32 {
    fn to_gm_version(&self) -> GMVersion {
        GMVersion::new(*self, 0, 0, 0, LTSBranch::PreLTS)
    }
}
