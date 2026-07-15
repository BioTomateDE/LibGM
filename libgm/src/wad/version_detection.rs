// SPDX-License-Identifier: GPL-3.0-only
mod acrv;
mod agrp;
mod bgnd;
mod code;
mod extn;
mod font;
mod func;
mod objt;
mod psem;
mod room;
mod sond;
mod sprt;
mod tgin;
mod txtr;

use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::chunk::ChunkName;
use crate::wad::elem::GMElement;
use crate::wad::parse::chunk::ChunkMap;
use crate::wad::parse::reader::DataReader;
use crate::wad::version::IdeVersion;

type CheckerFn = fn(&mut DataReader) -> Result<Option<GMVersion>>;

struct VersionCheck {
    /// The 4 letter name of the chunk where the check is performed.
    chunk_name: ChunkName,

    /// The function that performs the check.
    checker_fn: CheckerFn,

    /// The (lowest) WAD Version required for
    /// the checker to perform the detection.
    required_version: GMVersion,

    /// The (highest) WAD Version the checker can detect.
    target_version: GMVersion,
}

impl VersionCheck {
    // HACK: make this const when const traits are finally supported
    fn new(chunk: ChunkName, checker_fn: CheckerFn, req: GMVersion, target: GMVersion) -> Self {
        Self {
            chunk_name: chunk,
            checker_fn,
            required_version: req,
            target_version: target,
        }
    }
}

fn init_by_chunk_existence(chunks: &ChunkMap) -> Option<GMVersion> {
    const UPGRADES: [(ChunkName, GMVersion); 6] = [
        (ChunkName::UILR, GMVersion::GM2024_13),
        (ChunkName::PSEM, GMVersion::GM2023_2),
        (ChunkName::FEAT, GMVersion::GM2022_8),
        (ChunkName::FEDS, GMVersion::Studio2_3_6),
        (ChunkName::SEQN, GMVersion::Studio2_3),
        (ChunkName::TGIN, GMVersion::Studio2_2_1),
    ];

    for (chunk_name, version) in UPGRADES {
        if chunks.contains(chunk_name) {
            log::debug!("Existence of chunk {chunk_name} implies a Version of at least {version}");
            return Some(version);
        }
    }

    None
}

/// HACK: The `Into` trait is still not const unfortunately.
fn create_version_checks() -> [VersionCheck; 24] {
    [
        VersionCheck::new(
            ChunkName::SOND,
            sond::check_2024_6,
            GMVersion::GM2022_2,
            GMVersion::GM2024_6,
        ),
        VersionCheck::new(
            ChunkName::SPRT,
            sprt::check_2024_6,
            GMVersion::GM2022_2,
            GMVersion::GM2024_6,
        ),
        VersionCheck::new(
            ChunkName::FONT,
            font::check_2023_6_and_2024_11,
            GMVersion::GM2024_6,
            GMVersion::GM2024_11,
        ),
        VersionCheck::new(
            ChunkName::FONT,
            font::check_2023_6_and_2024_11,
            GMVersion::GM2022_8,
            GMVersion::Lts2022, // TODO: used to be 2023.6
        ),
        VersionCheck::new(
            ChunkName::ROOM,
            room::check_2022_1,
            GMVersion::Studio2_3,
            GMVersion::GM2022_1,
        ),
        VersionCheck::new(
            ChunkName::ROOM,
            room::check_2024_2_and_2024_4,
            GMVersion::GM2023_2,
            GMVersion::GM2024_4,
        ),
        VersionCheck::new(
            ChunkName::ROOM,
            room::check_2_2_2_302,
            GMVersion::Studio2,
            GMVersion::Studio2_2_2_302,
        ),
        VersionCheck::new(
            ChunkName::EXTN,
            extn::check_2022_6,
            GMVersion::Studio2_3,
            GMVersion::GM2022_6,
        ),
        VersionCheck::new(
            ChunkName::TXTR,
            txtr::check_2022_5,
            GMVersion::GM2022_3,
            GMVersion::GM2022_5,
        ),
        VersionCheck::new(
            ChunkName::TXTR,
            txtr::check_2022_3,
            GMVersion::Studio2_3,
            GMVersion::GM2022_6,
        ),
        VersionCheck::new(
            ChunkName::FONT,
            font::check_2024_14,
            GMVersion::GM2024_13,
            GMVersion::GM2024_14,
        ),
        VersionCheck::new(
            ChunkName::AGRP,
            agrp::check_2024_14,
            GMVersion::GM2024_13,
            GMVersion::GM2024_14,
        ),
        VersionCheck::new(
            ChunkName::EXTN,
            extn::check_2023_4,
            GMVersion::GM2022_6,
            GMVersion::GM2023_4,
        ),
        VersionCheck::new(
            ChunkName::TGIN,
            tgin::check_2023_1,
            GMVersion::GM2022_9,
            GMVersion::GM2023_1,
        ),
        VersionCheck::new(
            ChunkName::OBJT,
            objt::check_2022_5,
            GMVersion::Studio2_3,
            GMVersion::GM2022_5,
        ),
        VersionCheck::new(
            ChunkName::SPRT,
            sprt::check_2_3_2,
            GMVersion::Studio2,
            GMVersion::Studio2_3_2,
        ),
        VersionCheck::new(
            ChunkName::TGIN,
            tgin::check_2022_9,
            GMVersion::Studio2_3,
            GMVersion::GM2022_9,
        ),
        VersionCheck::new(
            ChunkName::TXTR,
            txtr::check_2_0_6,
            GMVersion::Studio2,
            GMVersion::Studio2_0_6,
        ),
        VersionCheck::new(
            ChunkName::PSEM,
            psem::check_2023_x,
            GMVersion::GM2023_2,
            GMVersion::GM2023_8,
        ),
        VersionCheck::new(
            ChunkName::ACRV,
            acrv::check_2_3_1,
            GMVersion::Studio2_3,
            GMVersion::Studio2_3_1,
        ),
        VersionCheck::new(
            ChunkName::BGND,
            bgnd::check_2024_14_1,
            GMVersion::GM2024_13,
            GMVersion::GM2024_14_1,
        ),
        VersionCheck::new(
            ChunkName::FUNC,
            func::check_2024_8,
            GMVersion::Wad14,
            GMVersion::GM2024_8,
        ),
        VersionCheck::new(
            ChunkName::CODE,
            code::check_2023_8_and_2024_4,
            GMVersion::Wad15,
            GMVersion::GM2024_4,
        ),
        VersionCheck::new(
            ChunkName::FONT,
            font::check_2022_2,
            GMVersion::Studio2_2_1,
            GMVersion::GM2022_2,
        ),
    ]
}

fn init_by_gen8(reader: &mut DataReader) -> Result<GMVersion> {
    reader.chunk = reader
        .chunks
        .get(ChunkName::GEN8)
        .ok_or("Chunk GEN8 does not exist")?;

    reader.cur_pos = reader.chunk.start_pos + 1; // Skip to WAD version
    let wad_version = reader.read_u8().ctx("reading WAD version")?;
    reader.cur_pos += 42; // Skip to IDE version
    let ide_version = IdeVersion::deserialize(reader).ctx("reading IDE version")?;

    log::debug!("GEN8 specifies IDE Version {ide_version} and WAD Version {wad_version}");

    if ide_version == IdeVersion::GMS2 {
        return Ok(GMVersion::Studio2);
    } else if reader.options.verify_constants {
        if ide_version.major != 1 {
            bail!("IDE Version {ide_version} does not have major 1 or 2");
        }
        if ide_version.minor != 0 || ide_version.release != 0 {
            bail!("GameMaker Studio 1 IDE Version {ide_version} has non-zero minor/release part");
        }
        if ide_version.build > 9999 {
            bail!("GameMaker Studio 1 IDE Version {ide_version} has a build greater than 9999");
        }
    }

    Ok(match wad_version {
        12 => GMVersion::Wad12,
        13 => GMVersion::Wad13,
        14 => GMVersion::Wad14,
        15 => GMVersion::Wad15,
        16 => GMVersion::Wad16Old,
        _ => bail!("Unknown WAD version {wad_version} and IDE version {ide_version}"),
    })
}

/// Games made in `GameMaker Studio 2` no longer store their actual version.
/// They only store `2.0.0.0`. In that case, the version needs to be detected
/// using assertions that can only be true in new versions.
/// Note that games which never use new features might be incorrectly detected
/// as an older version.
pub fn detect_format_version(mut reader: DataReader) -> Result<GMVersion> {
    reader.version = if let Some(version) = init_by_chunk_existence(&reader.chunks) {
        if reader.options.verify_constants {
            // read just to catch errors
            init_by_gen8(&mut reader).ctx("reading IDE and WAD version in GEN8")?;
        }
        version
    } else {
        init_by_gen8(&mut reader).ctx("reading IDE and WAD version in GEN8")?
    };
    log::debug!("Initialized version to {}", reader.version);

    // Nothing more to detect in Pre-GameMaker Studio 2
    if reader.version < GMVersion::Studio2 {
        return Ok(reader.version);
    }

    let checks: &[VersionCheck] = &create_version_checks();

    loop {
        let mut same_version: bool = true;

        for check in checks {
            // Skip checks that are already fulfilled.
            if reader.version >= check.target_version {
                continue;
            }

            // Skip versions whose version requirements are not *yet* met.
            if reader.version < check.required_version {
                continue;
            }

            // If chunk doesn't exist; just skip the check.
            let Some(chunk) = reader.chunks.get(check.chunk_name) else {
                continue;
            };

            reader.chunk = chunk;
            reader.cur_pos = reader.chunk.start_pos;

            let detected_version_opt: Option<GMVersion> =
                (check.checker_fn)(&mut reader).ctx(|| {
                    format!(
                        "detecting GameMaker Version {} in chunk {}",
                        check.target_version, check.chunk_name,
                    )
                })?;

            if let Some(detected_version) = detected_version_opt
                && reader.version < detected_version
            {
                log::debug!(
                    "Upgraded Version from {} to {} using check in chunk {}",
                    reader.version,
                    detected_version,
                    check.chunk_name,
                );
                reader.version = detected_version;
                same_version = false;
            }
        }

        if same_version {
            // Since it couldn't detect a higher version, there won't be any new checks
            // available that would now fulfil the minimum version requirement.
            break;
        }
    }

    Ok(reader.version)
}

macro_rules! target_version {
    ($ident:ident) => {
        Ok(Some($crate::wad::version::GMVersion::$ident))
    };
}
use target_version;
