[![Codeberg](https://img.shields.io/badge/codeberg-repo-blue?logo=codeberg)](https://codeberg.org/BioTomateDE/LibGM)
[![GitHub](https://img.shields.io/badge/github-repo-orchid?logo=github)](https://github.com/BioTomateDE/LibGM)
[![Crates.io](https://img.shields.io/crates/v/libgm)](https://crates.io/crates/libgm)
[![Documentation](https://img.shields.io/docsrs/libgm)](https://docs.rs/libgm)

# LibGM
A tool for unpacking, decompiling and modding GameMaker games such as Undertale or Deltarune.


## Benefits of this Rust port
- Parsing and building data files is ~8x faster than UndertaleModLib.
- Clean and maintainable library code.
- Thorough documentation on [docs.rs](https://docs.rs/libgm).
- Configurable lenient options for trying to parse half-broken data files (see `ParsingOptions`).
- Helpful error messages:
  - No `NullReferenceException`, ever!!!
  - No meaningless stack traces over 50 lines long.
  - Still more information than just "Reading out of bounds".
  - Strict data integrity checks catch errors earlier, making debugging easier.
  - Example trace printed out using `.chain()`:

This is an example error trace printed out using `error.chain()`:
```
sprite::swf::item::shape::style_group::fill::gradient::Record count 1065353216 implies data size 8.5 GB which exceeds failsafe size 10.0 MB
> while reading simple list
> while deserializing element 1/2 of sprite::swf::item::shape::style_group::StyleGroup<sprite::swf::item::subshape::Data> simple list
> while deserializing element 0/1 of sprite::swf::item::Item simple list
> while deserializing element 3/60 of GMSprite pointer list
> while deserializing chunk 'SPRT'
> while parsing GameMaker data file ./gm48_datafiles/a-loop_detective.win
```


## Disadvantages / TODOs
- GML Decompiler and Compiler not yet implemented. (Help would be greatly appreciated!)
- No edtior GUI yet, only a Rust library.


## Library usage
Add this line in the `[dependencies]` section of your `Cargo.toml` file:
```toml
libgm = "0.6"
```

Or if you want bleeding edge (might be unstable):
```toml
libgm = { git = "https://codeberg.org/BioTomateDE/LibGM" }
```


## Crate features
| Feature                 | Default  | Dependencies |
|-------------------------|----------|--------------|
| catch-panic             | enabled  |              |
| check-integrity         | enabled  |              |
| bzip2-image             | enabled  | bzip2        |
| png-image               | enabled  | image/png    |

- `catch-panic` catches panics in GameMaker (de)serialization functions
  and returns them as a LibGM error.
- `check-integrity` enables alignment and constant validation while parsing.
  These checks may still be demoted to a warning using `ParsingOptions`.
  Some checks regarding panic safety or memory allocation are always enabled.
- `bzip2-image` enables (de)serialization of BZip2+QOI encoded texture pages.
  If you try to change the format of a `GMImage` that stores BZip2-QOI
  data with this feature disabled, an error will be returned.
- `png-image` enables PNG (de)serialization.
  In games older than GM 2022.2, you will not be able to serialize `GMImage`s storing `DynamicImage`s with this feature disabled.


## Credits
Huge thanks to the Underminers Team! Without
[UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool),
this project would've been impossible.
I also want to thank the people in the Underminers Discord 
Guild who helped me along the way, especially
[@colinator27](https://github.com/colinator27).


## Licencing
This project is licenced under the
[GNU Public License v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html).
SPDX-License-Identifier: GPL-3.0-only.

This means that all projects using this library must also be licensed under GPL-3
to protect the Open Source Community.


## Contributing
All contributions are welcome! Whether that's a pull request, a feature you
would like to see added, a bug you found; just create an Issue/PR in this repo.
For more information, see the [contributing guidelines](https://codeberg.org/BioTomateDE/_/CONTRIBUTING.md).

- Everything related to GameMaker is located in `libgm/src/wad/`.
- A disassembler and assembler are available in `libgm/src/gml/assembly/`.
- The highly desired `Instruction` type is in `libgm/src/gml/instruction.rs`.
- There is a basic CLI to interact with LibGM in `libgm-cli/src/`.

### End-to-end testing
You must supply your own copy of specific versions of the games.
For instance, run the following commands (from the project root)
to copy from your Steam library folder, where applicable:

```bash
# Linux:
export STEAM="~/.steam/steam/steamapps/common"
# macOS:
export STEAM="~/Library/Application Support/Steam/steamapps/common"
# Windows (PowerShell):
$STEAM="C:/Program Files (x86)/Steam/steamapps/common"

# For Undertale:
cp $STEAM/Undertale/data.win libgm-cli/datafiles/undertaleXXX.win
# XXX needs to be the version: 100 for 1.00 | 101 for 1.001 | 108 for 1.08

# For Deltarune Chapters 1-4
cp $STEAM/DELTARUNE/data.win libgm-cli/resources/deltarune-launcher.win
cp $STEAM/DELTARUNE/chapter1_windows/data.win libgm-cli/resources/deltarune1.win
cp $STEAM/DELTARUNE/chapter2_windows/data.win libgm-cli/resources/deltarune2.win
cp $STEAM/DELTARUNE/chapter3_windows/data.win libgm-cli/resources/deltarune3.win
cp $STEAM/DELTARUNE/chapter4_windows/data.win libgm-cli/resources/deltarune4.win
```

Testing for Undertale 1.0.8 is gated behind the `test-undertale-XXX` feature (where XXX is the UT Version).
Deltarune Chapters 1-4 (v1.0.4) by `test-deltarune-ch1234`.
For example, if you have copied both in, run:

```bash
cargo t -plibgm-cli --features test-undertale-XXX,test-deltarune-ch1234
```
