// SPDX-License-Identifier: GPL-3.0-only
//! Contains GameMaker IDE Version types and abstractions to check and set versions.

use std::fmt::Display;
use std::fmt::Formatter;

use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

/// Different GameMaker release branches. LTS has some but not all features of
/// equivalent newer versions.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum LtsBranch {
    /// Before LTS was even introduced (`major < 2022`).
    Pre2022,

    /// The 2022 Long-Term Support branch.
    /// YoYoGames updates minor bugfixes here, but doesn't make breaking changes
    /// (except in 2023.6?).
    Lts2022,

    /// New Version but not the 2022 Long-Term Support branch.
    /// YoYo Games introduces all new features here, some of which may break
    /// your project.
    PostLts,
}

impl Display for LtsBranch {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let string = match self {
            Self::Pre2022 => "Pre LTS",
            Self::Lts2022 => "2022 LTS",
            Self::PostLts => "Post LTS",
        };
        f.write_str(string)
    }
}

/// A GameMaker Version, denoting the version of the IDE this game was created in.
///
/// This version struct is not updated by YoYo Games since GMS 2 and is
/// is stuck on `2.0.0.0` for modern versions.
/// If you need the format version of the data file, check out [`GMVersion`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdeVersion {
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
}

impl IdeVersion {
    pub const GMS2: Self = Self::new(2, 0, 0, 0);

    #[must_use]
    pub const fn new(major: u32, minor: u32, release: u32, build: u32) -> Self {
        Self { major, minor, release, build }
    }
}

impl GMElement for IdeVersion {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let major = reader.read_u32()?;
        let minor = reader.read_u32()?;
        let release = reader.read_u32()?;
        let build = reader.read_u32()?;
        Ok(Self { major, minor, release, build })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.major);
        builder.write_u32(self.minor);
        builder.write_u32(self.release);
        builder.write_u32(self.build);
        Ok(())
    }
}

impl Display for IdeVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let major = self.major;
        let minor = self.minor;
        let release = self.release;
        let build = self.build;

        if f.alternate() {
            return write!(f, "{major}.{minor}.{release}.{build}");
        }

        if major == 1 {
            return if let Some(minor) = gms1_minor_by_build(build) {
                write!(f, "1.{minor}.{build}")
            } else {
                write!(f, "1.X.{build}")
            };
        }

        write!(f, "{major}.{minor}")?;
        if release != 0 || build != 0 {
            write!(f, ".{release}")?;
            if build != 0 {
                write!(f, ".{build}")?;
            }
        }
        Ok(())
    }
}

/// The version of this GameMaker data file format.
///
/// This should correspond to the WAD ("bytecode") version specified in the `GEN8` chunk,
/// until GameMaker Studio 2, where they got lazy and stopped bumping it (stuck on WAD 17).
///
/// > NOTE: some of this information may be incorrect.
/// > I am not a GameMaker OG, I am desperately trying to find information on these old versions and documenting them.
///
/// Pre-GameMaker Studio versions (Mark Overmars times), such as 8.1, are not included here.
/// This GameMaker data file / WAD format was introduced in GameMaker Studio, which is what this library can parse and modify.
/// Previous (Pre-Studio) versions used a completly different way of storing game assets, and LibGM does not support them.
///
/// <https://web.archive.org/web/20150304025626/https://store.yoyogames.com/downloads/gm-studio/release-notes-studio.html>
///
/// GameMaker: Studio includes versions 1.1-1.4 and 2.0-2.3.
/// In the `GEN8` chunk, they stored the versions as `1.0.0.BUILD`,
/// where BUILD is the internal build number of that version (yes, the minor 1-4 is not written).
/// The majority of build numbers were not released. TThese numbers were bumped frequently in development, so they go into the hundereds.
/// They had a *stable* and *beta* branch: Beta releases were released as their actual build number,
/// whereas stable releases were relased as the build number plus 1000.
///
/// Then, they introduced GameMaker Studio 2, which changed lots of stuff
/// (least notably, removing the colon in "GameMaker: Studio").
/// They god rid of their shitty build-number versioning and actually had normal SemVer-like versioning for a while.
/// The last GameMaker Studio 2 version was 2.3 (i think) which also changed lots of stuff, most notably for GML.
/// Unfortunately, they also slowly stopped updating the WAD and IDE version fields in `GEN8`:
/// The IDE Version is now stuck on `2.0.0.0` forever.
/// The WAD version was stuck on 16 for a while. They bumped it one last time to 17, where it stayed stuck forever.
///
/// After that, they got rid of the "Studio" in the name and renamed it to just "GameMaker"
/// (same name as in Pre-Studio times, which is kind of confusing).
/// They switched their versioning system to the current `YYYY.MM.P.B`. Here is an excerpt from <https://gms-updates.gmclan.org/>:
/// > - YYYY - YEAR of release (2022 or higher)
/// > - MM - MONTH on which it was released (usually end of); beta versions are numbered as MONTH * 100 (not relevant for LibGM)
/// > - P - number of PATCH/fix (0 if it was first release)
/// > - B - total number of internal builds since May 2022 (last reset of build number was short before 2022.5 release, not every build is released to public)
/// > So, for example: 2022.6.1.53 means Update #1 for version released at end of June(6) 2022, 53rd build in total (since May 2022).
/// > There are no releases in July and December because of Holiday season peak.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd)]
#[non_exhaustive]
pub enum GMVersion {
    /// * WAD Version 12
    /// * GameMaker Studio 1.X.867
    /// Oldest known version.
    Wad12,

    /// * WAD Version 13
    /// * GameMaker Studio 1.4 Stable: 1135, 1451
    Wad13,

    /// * WAD Version 14
    /// * GameMaker Studio 1.4 Stable: 1464, 1474, 1499, 1567, 1598, 1657
    /// * GameMaker Studio 1.4 Beta: 420
    Wad14,

    /// * WAD Version 15
    /// * GameMaker Studio 1.4 Stable: 1683, 1690, 1711, 1747, 1749, 1750, 1763
    /// * GameMaker Studio 1.4 Beta: 460, 477, 533
    /// * GameMaker Studio 2.0.0.0 (?????? TODO: investigate)
    Wad15,

    /// * WAD Version 16
    /// * GameMaker Studio 1.4 Stable: 1539, 1767, 1772, 1773, 1778, 1804
    /// * GameMaker Studio 1.4 Beta: 551
    Wad16Old,

    /// * WAD Version 16
    /// * GameMaker Studio 1.4 Stable: 9999
    /// * Maybe Studio 2.0?
    /// Has padding bytes at the end of chunks
    Wad16Pad,

    /// * GameMaker Studio 2
    /// * WAD Version 16
    GMS2,

    /// * GameMaker Studio 2.0.6
    /// * WAD Version 16
    GMS2_0_6,

    // wad 17 beyond here
    /// * GameMaker Studio 2.2.1
    /// * WAD Version 17 (now forever)
    GMS2_2_1,
    GMS2_2_2_302,
    GMS2_3,
    GMS2_3_1,
    GMS2_3_2,
    GMS2_3_6,

    /// * GameMaker 2022.1
    /// * WAD Version 17
    GM2022_1,
    GM2022_2,
    GM2022_3,
    GM2022_5,
    GM2022_6,
    GM2022_8,

    /// * GameMaker 2022.9
    /// Same file format / features as initial 2022 LTS (2022.0.0).
    /// This is confirmed in <https://gamemaker.io/en/blog/release-2022-0#:~:text=2022%2E9>.
    GM2022_9,

    /// * GameMaker 2022 LTS: 2022.0.3
    /// **WARNING**: This is non-linear:
    /// this only introduces some features that 2022.9 - 2023.4 didn't have,
    /// but still lacks other features from 2023.1 - 2023.4.
    /// It is similar to Non-LTS 2023.6.
    ///
    /// TODO: what about 2022.0.1 and 2022.0.2 LTS?
    Lts2022_0_3,

    /// * GameMaker 2023.1
    /// * Introduces features that 2022.0.3 LTS lacks.
    GM2023_1,

    /// * GameMaker 2023.2
    /// * Introduces features that 2022.0.3 LTS lacks.
    GM2023_2,

    /// * GameMaker 2023.4
    /// * Introduces features that 2022.0.3 LTS lacks.
    GM2023_4,

    /// * GameMaker 2023.6
    /// Introduces "Line Height" for Fonts, which is also available in 2022.0.3 LTS.
    GM2023_6,

    GM2023_8,
    GM2023_11,
    GM2024_2,
    GM2024_4,
    GM2024_6,
    GM2024_8,
    GM2024_11,
    GM2024_13,
    GM2024_14,
    GM2024_14_1,
}

impl Display for GMVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GMVersion::Wad12 => "WAD 12",
            GMVersion::Wad13 => "WAD 13",
            GMVersion::Wad14 => "WAD 14",
            GMVersion::Wad15 => "WAD 15",
            GMVersion::Wad16Old => "WAD 16 (pre-GMS 2)",
            GMVersion::Wad16Pad => "WAD 16 (GMS 1.4.9999+)",
            GMVersion::GMS2 => "GMS 2",
            GMVersion::GMS2_0_6 => "GMS 2.0.6",
            GMVersion::GMS2_2_1 => "GMS 2.2.1",
            GMVersion::GMS2_2_2_302 => "GMS 2.2.2.302",
            GMVersion::GMS2_3 => "GMS 2.3",
            GMVersion::GMS2_3_1 => "GMS 2.3.1",
            GMVersion::GMS2_3_2 => "GMS 2.3.2",
            GMVersion::GMS2_3_6 => "GMS 2.3.6",
            GMVersion::GM2022_1 => "2022.1",
            GMVersion::GM2022_2 => "2022.2",
            GMVersion::GM2022_3 => "2022.3",
            GMVersion::GM2022_5 => "2022.5",
            GMVersion::GM2022_6 => "2022.6",
            GMVersion::GM2022_8 => "2022.8",
            GMVersion::GM2022_9 => "2022.9", // or 2022.0.0 LTS
            GMVersion::Lts2022_0_3 => "2022.0.3 LTS",
            GMVersion::GM2023_1 => "2023.1",
            GMVersion::GM2023_2 => "2023.2",
            GMVersion::GM2023_4 => "2023.4",
            GMVersion::GM2023_6 => "2022.6",
            GMVersion::GM2023_8 => "2023.8",
            GMVersion::GM2023_11 => "2023.11",
            GMVersion::GM2024_2 => "2024.2",
            GMVersion::GM2024_4 => "2024.4",
            GMVersion::GM2024_6 => "2024.6",
            GMVersion::GM2024_8 => "2024.8",
            GMVersion::GM2024_11 => "2024.11",
            GMVersion::GM2024_13 => "2024.13",
            GMVersion::GM2024_14 => "2024.14",
            GMVersion::GM2024_14_1 => "2024.14.1",
        };
        f.write_str(s)
    }
}

// TODO: better detection whether it's 1.1 - 1.4
fn gms1_minor_by_build(build: u32) -> Option<u8> {
    if build < 1000 || build > 9999 {
        return None;
    }
    if build >= 1451 {
        return Some(4);
    }
    None
}
