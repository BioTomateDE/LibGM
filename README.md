[![Codeberg](https://img.shields.io/badge/codeberg-repo-blue?logo=codeberg)](https://codeberg.org/BioTomateDE/LibGM)
[![GitHub](https://img.shields.io/badge/github-repo-orchid?logo=github)](https://github.com/BioTomateDE/LibGM)
[![Crates.io](https://img.shields.io/crates/v/libgm)](https://crates.io/crates/libgm)
[![Documentation](https://img.shields.io/docsrs/libgm)](https://docs.rs/libgm)

# LibGM
A tool for unpacking and modding GameMaker games such as Undertale or Deltarune.

## Benefits of this Rust port
- Parsing and building data files is ~8x faster than UndertaleModLib.
- Clean and maintainable library code.
- Thorough documentation on [docs.rs](https://docs.rs/libgm).
- Configurable lenient options for trying to load half-broken data files (see `LoadingOptions`).
- Helpful error messages:
  - No `NullReferenceException`, ever!!!
  - No meaningless stack traces over 50 lines long.
  - Still more information than just "Reading out of bounds".
  - Strict data integrity checks catch errors earlier, making debugging easier.

This is an example error trace printed out using `error.chain()`:

```
Invalid boolean value 4294967295 (0xFFFFFFFF) at position 51924
> while deserializing element 1/3 of sequence::track::keyframe::Keyframe<sequence::track::keyframe::color::Color> simple list
> while deserializing element 0/1 of sequence::track::Track simple list
> while deserializing element 0/1 of nullable sequence::Sequence pointer list
> while deserializing chunk SEQN
> while deserializing GameMaker data file gm48/sixty-seconds_do-it-fast.win
```

## What's supported
LibGM supports unpacking GameMaker games made between GameMaker: Studio 1 and GameMaker 2026.

GameMaker Classic (GM8 and older) has a completely different format.
You can use [OpenGMK](https://github.com/OpenGMK/OpenGMK) for these old games.

The new GMRT runtime (currently in beta) will also have a new format which LibGM will not support.

## Disadvantages / TODOs
- GML Decompiler and Compiler not yet implemented. (Help would be greatly appreciated!)
- No editor GUI yet, only a Rust library.

## Crate features
| Feature                 | Default  | Dependencies |
|-------------------------|----------|--------------|
| catch-panic             | enabled  |              |
| check-integrity         | enabled  |              |
| bzip2-image             | enabled  | bzip2        |
| png-image               | enabled  | image/png    |

- `catch-panic` catches panics in GameMaker (de)serialization functions
  and returns them as a LibGM error.
  (Panics can only occur when there is a bug in LibGM.)
- `check-integrity` enables alignment and constant validation while parsing.
  These checks may still be demoted to a warning using `LoadingOptions`.
  Some checks regarding panic safety or memory allocation are always enabled.
- `bzip2-image` enables (de)serialization of BZip2+QOI encoded texture pages.
  If you try to change the format of a `GMImage` that stores BZip2-QOI
  data with this feature disabled, an error will be returned.
- `png-image` enables PNG (de)serialization.
  In games older than GM 2022.2, you will not be able to serialize
  `GMImage`s storing `DynamicImage`s with this feature disabled.

## Credits
Huge thanks to the Underminers Team! Without
[UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool),
this project would've been impossible.
I also want to thank the people in the Underminers
Discord who helped me along the way, especially
[@colinator27](https://github.com/colinator27).

## Licencing
This project is licenced under the
[GNU Public License v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html).

This means that all projects using this library must also
be licensed under GPL-3 to protect the open source community.

## Contributing
All contributions are welcome! Whether that's a pull request, a feature you
would like to see added, a bug you found; just create an Issue/PR in this repo.
(Please do not submit AI-generated content.)

- GameMaker resource (de)serialization code is located in `libgm/src/wad/elem/`.
- Data file loading/saving code is located in `libgm/src/wad/load.rs` and `libgm/src/wad/save.rs`.
- A disassembler and assembler are available in `libgm/src/gml/assembly/`.
- The highly desired `Instruction` type is in `libgm/src/gml/instruction.rs`.
- There is a basic CLI to interact with LibGM in `libgm-cli/src/`.
