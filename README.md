[![GitHub](https://img.shields.io/badge/github-repo-blue?logo=github)](https://github.com/BioTomateDE/LibGM)
[![Crates.io](https://img.shields.io/crates/v/libgm)](https://crates.io/crates/libgm)
[![Documentation](https://img.shields.io/docsrs/libgm)](https://docs.rs/libgm)

# LibGM

A tool for unpacking, decompiling and modding GameMaker games such as Undertale
or Deltarune.

This is effectively a Rust port of
[UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool)
(specifically UndertaleModLib).

## Benefits of this Rust port

- Runtime for parsing and building is ~8x faster than UndertaleModLib.
  - This can be further accelerated with the upcoming threaded chunk reading
    feature (opt-in).
- Thorough documentation on docs.rs.
- Clean and maintainable library code.
- Helpful error messages:
  - No `NullReferenceException`, ever
  - No meaningless stack traces over 50 lines long
  - Still more information than just "Reading out of bounds"
  - Strict data integrity checks catch errors earlier, making debugging easier
  - Example trace printed out using `.chain_pretty()`:
    ```
    sprite::swf::item::shape::style_group::fill::gradient::Record count 1065353216 implies data size 8.5 GB which exceeds failsafe size 10.0 MB
    ↳ while reading simple list
    ↳ while deserializing element 1/2 of sprite::swf::item::shape::style_group::StyleGroup<sprite::swf::item::subshape::Data> simple list
    ↳ while deserializing element 0/1 of sprite::swf::item::Item simple list
    ↳ while deserializing element 3/60 of GMSprite pointer list
    ↳ while deserializing chunk 'SPRT'
    ↳ while parsing GameMaker data file ./gm48_datafiles/a-loop_detective.win
    ```
- Configurable lenient options for trying to parse half-broken data files.

## Disadvantages / TODOs

- Null pointers are not yet supported.
- GML Decompiler and Compiler not yet implemented (help would be greatly
  appreciated!)
- No GUI yet, only a Rust library.

## How to use as a dependency

Add this line in the `[dependencies]` section of your `Cargo.toml` file:

```toml
libgm = "0.2.0"
```

Or if you want bleeding edge:

```toml
libgm = { git = "https://github.com/BioTomateDE/LibGM" }
```

Now you can use these functions exposed by LibGM:

- `parse_file(data_file_path: impl AsRef<Path>) -> Result<GMData>`
- `parse_bytes(raw_data: impl AsRef<[u8]>) -> Result<GMData>`
- `build_data(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()>`
- `build_bytes(gm_data: &GMData) -> Result<Vec<u8>>`

If you need more control over how the data file should be read, you can also use
the `DataParserOptions` struct to modify parsing options:

```rust
// Create a parser with custom options
let parser = DataParserOptions::new()
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

// Parse from a byte slice reference
let byte_slice: &[u8] = &[0x46, 0x4F, 0x52, 0x4D, /* ... */];
let data: GMData = parser.parse_bytes(byte_slice)?;

// You can also parse directly from borrowed data
let buffer: Vec<u8> = std::fs::read("./data.win")?;
let data: GMData = parser.parse_bytes(&buffer)?;
// buffer is still accessible here since we passed a reference
```

## Credits

Huge thanks to the Underminers Team! Without UndertaleModTool, this project
would've been impossible. I also want to thank the people in the Underminers
Discord Server who helped me along the way, especially
[@colinator27](https://github.com/colinator27).

## Licencing

This project is licenced under the
[GNU Public Licence v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html) (GPL-3).

## Contributing

All contributions are welcome! Whether that's a pull request, a feature you
would like to see added, a bug you found; just create an Issue/PR in this repo.

- Everything related to GameMaker is located in `libgm/src/gamemaker/`.
- There is a basic CLI to interact with LibGM. Its code is located in
  `libgm-cli/src/`.

## Roadmap
[ ] Add QOI and Bz2Qoi image serialization
[ ] Implement threading for parser
[ ] Add crate features (maybe use prefix for dependency disablers?):
  [ ] bzip2 (opt-out): Enables Bz2Qoi image support
  [ ] png (opt-out: Enables PNG image support
  [ ] uuid (opt-out): Exposes the general info uuid field. stored as raw data otherwise
  [ ] chrono (opt-out): Exposes the general info creation timestamp field. stored as raw data otherwise
  [ ] integrity-checks (opt-out): Enables all data integrity checks (pointer validation, constant validation etc). These checks may still be demoted to a warning using ParsingOptions. Some checks regarding panic safety or memory allocation should always be enabled.
  [ ] panic-catching (opt-out): Sets a panic handler before data [de]serialization, returning a LibGM error if a panic occurred
[ ] Overhaul the CLI: Allow for viewing of relevant data, exporting assembly and more
[ ] Maybe move the CLI to a different repo / publish it?
[ ] Add helpers to Instruction (like `fn get_variable(&self) -> Option<&CodeVariable>` that returns some for push, pushloc, pushglb, pushbltn or pop)
[ ] "Fix" disassembler for child code entries (right now they will generate empty string)
[ ] Maybe add some sort of header for assembly for entire code entries (name, local count, arg count) so u can assemble_code directly
[ ] Add comments to assembly? Which style tho? How much does it fuck Up efficiency and maintainability?