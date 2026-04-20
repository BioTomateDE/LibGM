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
use crate::wad::deserialize::chunk::ChunkBounds;
use crate::wad::deserialize::chunk::ChunkMap;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::version::LTSBranch::LTS;
use crate::wad::version::LTSBranch::PostLTS;
use crate::wad::version::LTSBranch::PreLTS;
use crate::wad::version::ToGMVersion;

/// If `check_fn` can detect multiple versions, `required_version` should be set
/// to its _lowest_ required version whereas `target_version` should be set to
/// the _highest_ possible version it can detect.
fn try_check(
    reader: &mut DataReader,
    chunk: &'static str,
    check_fn: fn(&mut DataReader) -> Result<Option<GMVersion>>,
    target_version: impl ToGMVersion,
) -> Result<()> {
    let chunk_name = ChunkName::new(chunk);
    let target_version = target_version.to_gm_version();

    // Return if highest possible detected version is already fulfilled.
    if reader.general_info.version >= target_version {
        return Ok(());
    }

    // Return if the chunk does not exist.
    let Some(chunk) = reader.chunks.get_by_name(chunk_name) else {
        return Ok(());
    };

    reader.chunk = chunk.clone();
    reader.cur_pos = chunk.start_pos;

    // Detect the version.
    let version_req_opt = check_fn(reader).with_context(|| {
        format!("manually detecting GameMaker Version {target_version} in chunk {chunk_name:?}")
    })?;

    // Return if no version could be detected.
    let Some(version_req) = version_req_opt else {
        return Ok(());
    };

    // Return if the detected version is already fulfilled.
    if reader.general_info.version >= version_req {
        return Ok(());
    }

    log::debug!(
        "Upgraded Version from {} to {} using manual check in chunk '{}'",
        reader.general_info.version,
        version_req,
        chunk_name,
    );
    reader.general_info.set_version(version_req);

    Ok(())
}

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
    fn new(
        chunk: &'static str,
        checker_fn: CheckerFn,
        req: impl ToGMVersion,
        target: impl ToGMVersion,
    ) -> Self {
        // TODO(const): make this const when const traits are finally supported
        Self {
            chunk_name: ChunkName::new(chunk),
            checker_fn,
            required_version: req.to_gm_version(),
            target_version: target.to_gm_version(),
        }
    }
}

fn upgrade_by_chunk_existence(chunks: &ChunkMap) -> Option<GMVersion> {
    const UPGRADES: [(&str, GMVersion); 6] = [
        ("UILR", GMVersion::new(2024, 13, 0, 0, PostLTS)),
        ("PSEM", GMVersion::new(2023, 2, 0, 0, PostLTS)),
        ("FEAT", GMVersion::new(2022, 8, 0, 0, PreLTS)),
        ("FEDS", GMVersion::new(2, 3, 6, 0, PreLTS)),
        ("SEQN", GMVersion::new(2, 3, 0, 0, PreLTS)),
        ("TGIN", GMVersion::new(2, 2, 1, 0, PreLTS)),
    ];

    for (chunk_name, version) in UPGRADES {
        if chunks.contains(chunk_name) {
            log::debug!(
                "Existence of chunk '{chunk_name}' implies a Version of at least {version}"
            );
            return Some(version);
        }
    }

    None
}

/// TODO(const-hack): The `Into` trait is still not const unfortunately.
fn create_version_checks() -> [VersionCheck; 21] {
    [
        VersionCheck::new("SOND", sond::check_2024_6, (2022, 2, PostLTS), (2024, 6)),
        VersionCheck::new("SPRT", sprt::check_2024_6, (2022, 2, PostLTS), (2024, 6)),
        VersionCheck::new(
            "FONT",
            font::check_2023_6_and_2024_11,
            (2024, 6),
            (2024, 11),
        ),
        VersionCheck::new("FONT", font::check_2023_6_and_2024_11, (2022, 8), (2023, 6)),
        VersionCheck::new("ROOM", room::check_2022_1, (2, 3), (2022, 1)),
        VersionCheck::new("ROOM", room::check_2024_2_and_2024_4, (2023, 2), (2024, 4)),
        VersionCheck::new("ROOM", room::check_2_2_2_302, 2, (2, 2, 2, 302)),
        VersionCheck::new("EXTN", extn::check_2022_6, (2, 3), (2022, 6)),
        VersionCheck::new("TXTR", txtr::check_2022_5, (2022, 3), (2022, 5)),
        VersionCheck::new("TXTR", txtr::check_2022_3, (2, 3), (2022, 3)),
        VersionCheck::new("FONT", font::check_2024_14, (2024, 13), (2024, 14)),
        VersionCheck::new("AGRP", agrp::check_2024_14, (2024, 13), (2024, 14)),
        VersionCheck::new("EXTN", extn::check_2023_4, (2022, 6), (2023, 4)),
        VersionCheck::new("TGIN", tgin::check_2023_1, (2022, 9), (2023, 1)),
        VersionCheck::new("OBJT", objt::check_2022_5, (2, 3), (2022, 5)),
        VersionCheck::new("SPRT", sprt::check_2_3_2, 2, (2, 3, 2)),
        VersionCheck::new("TGIN", tgin::check_2022_9, (2, 3), (2022, 9)),
        VersionCheck::new("TXTR", txtr::check_2_0_6, 2, (2, 0, 6)),
        VersionCheck::new("PSEM", psem::check_2023_x, (2023, 2), (2023, 8)),
        VersionCheck::new("ACRV", acrv::check_2_3_1, (2, 3), (2, 3, 1)),
        VersionCheck::new("BGND", bgnd::check_2024_14_1, (2024, 13), (2024, 14, 1)),
    ]
}

/// Games made in `GameMaker Studio 2` no longer store their actual version.
/// They only store `2.0.0.0`. In that case, the version needs to be detected
/// using assertions that can only be true in new versions.
/// Note that games which never use new features might be incorrectly detected
/// as an older version.
pub fn detect_gamemaker_version(reader: &mut DataReader) -> Result<()> {
    let saved_pos = reader.cur_pos;
    let saved_chunk: ChunkBounds = reader.chunk.clone();

    if let Some(version) = upgrade_by_chunk_existence(&reader.chunks) {
        reader.general_info.set_version(version);
    }

    if reader.general_info.wad_version >= 14 {
        try_check(reader, "FUNC", func::check_2024_8, (2024, 8))?;
    }
    if reader.general_info.wad_version >= 15 {
        try_check(reader, "CODE", code::check_2023_8_and_2024_4, (2024, 4))?;
    }
    if reader.general_info.wad_version >= 17 {
        try_check(reader, "FONT", font::check_2022_2, (2022, 2))?;
    }

    let checks: &[VersionCheck] = &create_version_checks();

    loop {
        let mut same_version: bool = true;

        for check in checks {
            // Skip checks that are already fulfilled.
            // (You can comment this out for debug purposes to verify
            // that all version checks work properly.)
            if reader.general_info.version >= check.target_version {
                continue;
            }

            // Skip versions whose version requirements are not *yet* met.
            if reader.general_info.version < check.required_version {
                continue;
            }

            // If chunk doesn't exist; just skip the check.
            let Some(chunk) = reader.chunks.get_by_name(check.chunk_name) else {
                continue;
            };

            reader.chunk = chunk.clone();
            reader.cur_pos = reader.chunk.start_pos;

            let detected_version_opt: Option<GMVersion> =
                (check.checker_fn)(reader).with_context(|| {
                    format!(
                        "detecting GameMaker Version {} in chunk '{}'",
                        check.target_version, check.chunk_name,
                    )
                })?;

            if let Some(detected_version) = detected_version_opt
                && reader.general_info.version < detected_version
            {
                log::debug!(
                    "Upgraded Version from {} to {} using check in chunk '{}'",
                    reader.general_info.version,
                    detected_version,
                    check.chunk_name,
                );
                reader.general_info.set_version(detected_version);
                same_version = false;
            }
        }

        if same_version {
            // Since it couldn't detect a higher version, there won't be any new checks
            // available that would now fulfil the minimum version requirement.
            break;
        }
    }

    // Set the LTS branch properly.
    let ver: &mut GMVersion = &mut reader.general_info.version;
    if *ver >= (2023, 1) && ver.branch == PreLTS {
        ver.branch = LTS;
    }

    reader.cur_pos = saved_pos;
    reader.chunk = saved_chunk;
    Ok(())
}

macro_rules! target_version {
    ($($part:expr),+ $(,)?) => {
        Ok(Some($crate::wad::version::ToGMVersion::into_gm_version(($($part,)+))))
    };
}
use target_version;
