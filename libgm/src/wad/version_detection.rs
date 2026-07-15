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
use crate::wad::GMVersion::*;
use crate::wad::chunk::ChunkName;
use crate::wad::chunk::ChunkName::*;
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
        (UILR, GM2024_13),
        (PSEM, GM2023_2),
        (FEAT, GM2022_8),
        (FEDS, GMS2_3_6),
        (SEQN, GMS2_3),
        (TGIN, GMS2_2_1),
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
        VersionCheck::new(SOND, sond::check_2024_6, GM2022_2, GM2024_6),
        VersionCheck::new(SPRT, sprt::check_2024_6, GM2022_2, GM2024_6),
        VersionCheck::new(FONT, font::check_2023_6_and_2024_11, GM2024_6, GM2024_11),
        VersionCheck::new(FONT, font::check_2023_6_and_2024_11, GM2022_8, GM2024_11),
        VersionCheck::new(ROOM, room::check_2022_1, GMS2_3, GM2022_1),
        VersionCheck::new(ROOM, room::check_2024_2_and_2024_4, GM2023_2, GM2024_4),
        VersionCheck::new(ROOM, room::check_2_2_2_302, GMS2, GMS2_2_2_302),
        VersionCheck::new(EXTN, extn::check_2022_6, GMS2_3, GM2022_6),
        VersionCheck::new(TXTR, txtr::check_2022_5, GM2022_3, GM2022_5),
        VersionCheck::new(TXTR, txtr::check_2022_3, GMS2_3, GM2022_6),
        VersionCheck::new(FONT, font::check_2024_14, GM2024_13, GM2024_14),
        VersionCheck::new(AGRP, agrp::check_2024_14, GM2024_13, GM2024_14),
        VersionCheck::new(EXTN, extn::check_2023_4, GM2022_6, GM2023_4),
        VersionCheck::new(TGIN, tgin::check_2023_1, GM2022_9, GM2023_1),
        VersionCheck::new(OBJT, objt::check_2022_5, GMS2_3, GM2022_5),
        VersionCheck::new(SPRT, sprt::check_2_3_2, GMS2, GMS2_3_2),
        VersionCheck::new(TGIN, tgin::check_2022_9, GMS2_3, GM2022_9),
        VersionCheck::new(TXTR, txtr::check_2_0_6, GMS2, GMS2_0_6),
        VersionCheck::new(PSEM, psem::check_2023_x, GM2023_2, GM2023_8),
        VersionCheck::new(ACRV, acrv::check_2_3_1, GMS2_3, GMS2_3_1),
        VersionCheck::new(BGND, bgnd::check_2024_14_1, GM2024_13, GM2024_14_1),
        VersionCheck::new(FUNC, func::check_2024_8, Wad14, GM2024_8),
        VersionCheck::new(CODE, code::check_2023_8_and_2024_4, Wad15, GM2024_4),
        VersionCheck::new(FONT, font::check_2022_2, GMS2_2_1, GM2022_2),
    ]
}

fn init_by_gen8(reader: &mut DataReader) -> Result<GMVersion> {
    reader.chunk = reader.chunks.get(GEN8).ok_or("Chunk GEN8 does not exist")?;

    reader.cur_pos = reader.chunk.start_pos + 1; // Skip to WAD version
    let wad_version = reader.read_u8().ctx("reading WAD version")?;
    reader.cur_pos += 42; // Skip to IDE version
    let ide_version = IdeVersion::deserialize(reader).ctx("reading IDE version")?;

    log::debug!("GEN8 specifies IDE Version {ide_version} and WAD Version {wad_version}");

    if ide_version == IdeVersion::GMS2 {
        return Ok(GMS2);
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
        12 => Wad12,
        13 => Wad13,
        14 => Wad14,
        15 => Wad15,
        16 => Wad16Old,
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
    if reader.version < GMS2 {
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
                if detected_version == Lts2022_0_3 {
                    return Ok(Lts2022_0_3);
                }
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
