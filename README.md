[![Codeberg](https://img.shields.io/badge/codeberg-repo-blue?logo=codeberg)](https://codeberg.org/BioTomateDE/LibGM)
[![GitHub](https://img.shields.io/badge/github-repo-orchid?logo=github)](https://github.com/BioTomateDE/LibGM)
[![Crates.io](https://img.shields.io/crates/v/libgm)](https://crates.io/crates/libgm)
[![Documentation](https://img.shields.io/docsrs/libgm)](https://docs.rs/libgm)

# LibGM
A tool for unpacking, decompiling and modding GameMaker games such as Undertale
or Deltarune.

> This project migrated from GitHub to
[Codeberg](https://codeberg.org/BioTomateDE/LibGM).
Please open issues and pull requests on Codeberg, if possible.

## Benefits of this Rust port
- Parsing and building data files is ~8x faster than UndertaleModLib.
- Clean and maintainable library code.
- Thorough documentation on [docs.rs](https://docs.rs).
- Helpful error messages:
  - No `NullReferenceException`, ever
  - No meaningless stack traces over 50 lines long
  - Still more information than just "Reading out of bounds"
  - Strict data integrity checks catch errors earlier, making debugging easier
  - Example trace printed out using `.chain()`:
    ```
    sprite::swf::item::shape::style_group::fill::gradient::Record count 1065353216 implies data size 8.5 GB which exceeds failsafe size 10.0 MB
    > while reading simple list
    > while deserializing element 1/2 of sprite::swf::item::shape::style_group::StyleGroup<sprite::swf::item::subshape::Data> simple list
    > while deserializing element 0/1 of sprite::swf::item::Item simple list
    > while deserializing element 3/60 of GMSprite pointer list
    > while deserializing chunk 'SPRT'
    > while parsing GameMaker data file ./gm48_datafiles/a-loop_detective.win
    ```

- Configurable lenient options for trying to parse half-broken data files (see `ParsingOptions`).

## Disadvantages / TODOs
- Null pointers are not yet supported.
- GML Decompiler and Compiler not yet implemented (help would be greatly appreciated!)
- No GUI yet, only a Rust library.

## How to use as a dependency
Add this line in the `[dependencies]` section of your `Cargo.toml` file:

```toml
libgm = "0.5"
```

Or if you want bleeding edge:

```toml
libgm = { git = "https://codeberg.org/BioTomateDE/LibGM" }
```

Now you can use these functions exposed by LibGM:

- `parse_file(data_file_path: impl AsRef<Path>) -> Result<GMData>`
- `parse_bytes(raw_data: impl AsRef<[u8]>) -> Result<GMData>`
- `build_data(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()>`
- `build_bytes(gm_data: &GMData) -> Result<Vec<u8>>`

If you need more control over how the data file should be read, you can also use
the `ParsingOptions` struct to modify parsing options:

```rust
// Create a parser with custom options
let parser = ParsingOptions::new()
    .verify_alignment(false)
    .allow_unread_chunks(true);

// Parse multiple files
for path in data_files {
    let data: GMData = parser.parse_file(path)?;
    // Process the parsed data...
}

// Parse from a byte vector
let raw_data: Vec<u8> = read_from_zip(zip_file, "Undertale/assets/game.unx")?;
let data: GMData = parser.parse_bytes(raw_data)?;
```

## Crate features
| Feature                 | Default  | Dependencies |
|-------------------------|----------|--------------|
| catch-panic             | enabled  |              |
| check-integrity         | enabled  |              |
| game-creation-timestamp | disabled | chrono       |
| bzip2-image             | enabled  | bzip2        |
| png-image               | enabled  | image/png    |

- `catch-panic` catches panics in GameMaker (de)serialization functions
  and returns them as a LibGM error.
- `check-integrity` enables alignment and constant validation while parsing.
  These checks may still be demoted to a warning using `ParsingOptions`.
  Some checks regarding panic safety or memory allocation are always enabled.
- `game-creation-timestamp` exposes the `creation-timestamp` field in `GMGeneralInfo`.
  Otherwise, it will be stored as an i64 internally.
- `bzip2-image` enables (de)serialization of BZip2+QOI encoded texture pages.
  If you try to change the format of a `GMImage` that stores BZip2-QOI
  data with this feature disabled, an error will be returned.
- `png-image` enables PNG (de)serialization.
  In games older than GM 2022.2, you will not be able to serialize `GMImage`s storing `DynamicImage`s with this feature disabled.

## Credits
Huge thanks to the Underminers Team! Without
[UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool),
this project
would've been impossible. I also want to thank the people in the Underminers
Discord Guild who helped me along the way, especially
[@colinator27](https://github.com/colinator27).

## Licencing
This project is licenced under the
[GNU Public License v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html) (GPL-3.0).

This means that all projects using this library must also be licensed under GPL-3.

## Contributing
All contributions are welcome! Whether that's a pull request, a feature you
would like to see added, a bug you found; just create an Issue/PR in this repo.

- Everything related to GameMaker is located in `libgm/src/wad/`.
- A disassembler and assembler are available in `libgm/src/gml/assembly/`.
- There is a basic CLI to interact with LibGM in `libgm-cli/src/`.
