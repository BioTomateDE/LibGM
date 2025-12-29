mod acrv;
mod agrp;
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

use crate::{
    gamemaker::{
        chunk::ChunkName,
        deserialize::{
            chunk::{ChunkBounds, ChunkMap},
            reader::DataReader,
        },
        version::{
            GMVersionReq,
            LTSBranch::{LTS, PostLTS, PreLTS},
        },
    },
    prelude::*,
};

/// If `check_fn` can detect multiple versions, `required_version` should be set to its _lowest_ required version
/// whereas `target_version` should be set to the _highest_ possible version it can detect.
fn try_check<V: Into<GMVersionReq>>(
    reader: &mut DataReader,
    chunk: &'static str,
    check_fn: fn(&mut DataReader) -> Result<Option<GMVersionReq>>,
    target_version: V,
) -> Result<()> {
    let chunk_name = ChunkName::new(chunk);
    let target_version: GMVersionReq = target_version.into();
    if reader.general_info.is_version_at_least(target_version) {
        return Ok(()); // No need to check if already
    }

    let Some(chunk) = reader.chunks.get_by_name(chunk_name) else {
        return Ok(());
    };
    reader.chunk = chunk.clone();
    reader.cur_pos = chunk.start_pos;

    let Some(version_req) = check_fn(reader)? else {
        return Ok(());
    };
    if reader.general_info.is_version_at_least(version_req.clone()) {
        return Ok(());
    }
    log::debug!(
        "Upgraded Version from {} to {} using manual check in chunk '{}'",
        reader.general_info.version,
        version_req,
        chunk_name,
    );
    reader.general_info.set_version_at_least(version_req)?;

    Ok(())
}

type CheckerFn = fn(&mut DataReader) -> Result<Option<GMVersionReq>>;

struct VersionCheck {
    /// The 4 letter name of the chunk where the check is performed.
    chunk_name: ChunkName,

    /// The function that performs the check.
    checker_fn: CheckerFn,

    /// The (lowest) gamemaker version required for
    /// the checker to perform the detection.
    required_version: GMVersionReq,
    /// The (highest) gamemaker version the checker can detect.
    ///
    target_version: GMVersionReq,
}

impl VersionCheck {
    fn new<R: Into<GMVersionReq>, V: Into<GMVersionReq>>(
        chunk: &'static str,
        checker_fn: CheckerFn,
        req: R,
        target: V,
    ) -> Self {
        Self {
            chunk_name: ChunkName::new(chunk),
            checker_fn,
            required_version: req.into(),
            target_version: target.into(),
        }
    }
}

fn upgrade_by_chunk_existence(chunks: &ChunkMap) -> Option<GMVersionReq> {
    const UPGRADES: [(&str, GMVersionReq); 6] = [
        ("UILR", GMVersionReq::new(2024, 13, 0, 0, PostLTS)),
        ("PSEM", GMVersionReq::new(2023, 2, 0, 0, PostLTS)),
        ("FEAT", GMVersionReq::new(2022, 8, 0, 0, PreLTS)),
        ("FEDS", GMVersionReq::new(2, 3, 6, 0, PreLTS)),
        ("SEQN", GMVersionReq::new(2, 3, 0, 0, PreLTS)),
        ("TGIN", GMVersionReq::new(2, 2, 1, 0, PreLTS)),
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
fn create_version_checks() -> Vec<VersionCheck> {
    vec![
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
        VersionCheck::new("ROOM", room::check_2_2_2_302, (2, 0), (2, 2, 2, 302)),
        VersionCheck::new("EXTN", extn::check_2022_6, (2, 3), (2022, 6)),
        VersionCheck::new("TXTR", txtr::check_2022_5, (2022, 3), (2022, 5)),
        VersionCheck::new("TXTR", txtr::check_2022_3, (2, 3), (2022, 3)),
        VersionCheck::new("FONT", font::check_2024_14, (2024, 13), (2024, 14)),
        VersionCheck::new("AGRP", agrp::check_2024_14, (2024, 13), (2024, 14)),
        VersionCheck::new("EXTN", extn::check_2023_4, (2022, 6), (2023, 4)),
        VersionCheck::new("TGIN", tgin::check_2023_1, (2022, 9), (2023, 1)),
        VersionCheck::new("OBJT", objt::check_2022_5, (2, 3), (2022, 5)),
        VersionCheck::new("SPRT", sprt::check_2_3_2, (2, 0), (2, 3, 2)),
        VersionCheck::new("TGIN", tgin::check_2022_9, (2, 3), (2022, 9)),
        VersionCheck::new("TXTR", txtr::check_2_0_6, (2, 0), (2, 0, 6)),
        VersionCheck::new("PSEM", psem::check_2023_x, (2023, 2), (2023, 8)),
        VersionCheck::new("ACRV", acrv::check_2_3_1, (2, 3), (2, 3, 1)),
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
        reader.general_info.set_version_at_least(version)?;
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

    let mut checks: Vec<VersionCheck> = create_version_checks();

    loop {
        // Permanently filter out already detected versions
        checks.retain(|i| {
            !reader
                .general_info
                .is_version_at_least(i.target_version.clone())
        });

        let mut updated_version: bool = false;
        let mut checks_to_remove: Vec<bool> = vec![false; checks.len()];

        for (i, check) in checks.iter().enumerate() {
            // For this iteration, filter out versions whose version requirements are not met yet
            if !reader
                .general_info
                .is_version_at_least(check.required_version.clone())
            {
                continue;
            }

            // Permanently remove check; no matter if successful or not
            checks_to_remove[i] = true;

            // If chunk doesn't exist; just skip the check
            let Some(chunk) = reader.chunks.get_by_name(check.chunk_name) else {
                continue;
            };

            reader.chunk = chunk.clone();
            reader.cur_pos = reader.chunk.start_pos;

            let detected_version_opt: Option<GMVersionReq> = (check.checker_fn)(reader)
                .with_context(|| {
                    format!(
                        "trying to detect GameMaker Version {} in chunk '{}'",
                        check.target_version, check.chunk_name,
                    )
                })?;

            if let Some(detected_version) = detected_version_opt {
                log::debug!(
                    "Upgraded Version from {} to {} using check in chunk '{}'",
                    reader.general_info.version,
                    detected_version,
                    check.chunk_name,
                );
                reader.general_info.set_version_at_least(detected_version)?;
                updated_version = true;
            }
        }

        // Remove all performed checks
        for (i, should_remove) in checks_to_remove.into_iter().enumerate().rev() {
            if should_remove {
                checks.remove(i);
            }
        }

        if !updated_version {
            // Since it couldn't detect a higher version, there won't be any new checks
            // available that would now fulfil the minimum version requirement.
            break;
        }
    }

    if reader.general_info.is_version_at_least((2023, 1))
        && reader.general_info.version.branch == PreLTS
    {
        reader.general_info.version.branch = LTS;
    }

    reader.cur_pos = saved_pos;
    reader.chunk = saved_chunk;
    Ok(())
}
