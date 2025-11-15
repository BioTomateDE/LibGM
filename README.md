# LibGM

A tool for unpacking, decompiling and modding GameMaker games such as Undertale
or Deltarune.

This is effectively a Rust port of
[UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool)
(specifically UndertaleModLib and
[Underanalyzer](https://github.com/UnderminersTeam/Underanalyzer)).

## Benefits of this Rust port

- Runtime for parsing and building is ~8x faster than UndertaleModLib.
  - This can be further accelerated with the upcoming threaded chunk reading
    feature (opt-in).
- Thorough documentation on docs.rs (half todo).
- Clean and maintainable library code.
- Helpful error messages:
  - No `NullReferenceException`, ever
  - No useless stack traces over 30 lines long
  - Strict data integrity checks catch errors earlier, making debugging easier
  - Example trace printed out using `.chain()`:
    ```
    Invalid GMSpriteSepMaskType 67 (0x00000043)
    ↳ while deserializing element 273/4437 of GMSprite pointer list
    ↳ while deserializing chunk 'SPRT'
    ↳ while parsing GameMaker data file "data.win"
    ```
- Configurable lenient options for trying to parse half-broken data files.

## Disadvantages / TODOs

- No GUI yet, only a Rust library.
- Null pointers are not yet supported.
- Decompiler and compiler not nearly done.

## How to use as a dependency

Add this line in the `[dependencies]` section of your `Cargo.toml` file:

```toml
libgm = { git = "https://github.com/BioTomateDE/LibGM" }
```

_This crate will also be added to [crates.io](https://https://crates.io/) when
it is finished._

Now you can use these functions exposed by LibGM:

- `parse_data_file(data_file_path: impl AsRef<Path>) -> Result<GMData>`
- `fn build_data_file(gm_data: &GMData) -> Result<Vec<u8>>`
- `write_data_file(gm_data: &GMData, path: impl AsRef<Path>) -> Result<()>`
- `decompile_to_ast()` (WIP)

If you need more control over how the data file should be read, you can also use
the `DataParser` struct to modify parsing options:

```rust
// Create a parser with custom options
let parser = DataParser::new()
    .verify_alignment(false)
    .parallel_processing(true);

// Parse multiple files
for path in data_files {
    let data: GMData = parser.parse_file(path)?;
    // Process the parsed data...
}

// Parse from a byte vector
let raw_data: Vec<u8> = read_from_zip(zip_file, "env/game.unx")?;
let data: GMData = parser.parse_bytes(raw_data)?;

// Parse from a byte slice reference
let byte_slice: &[u8] = &[0x46, 0x4F, 0x52, 0x4D, /* ... */];
let data: GMData = parser.parse_bytes(byte_slice)?;

// You can also parse directly from borrowed data
let buffer: Vec<u8> = std::fs::read("game.win")?;
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

- Everything related to GameMaker is located in `src/libgm/gamemaker/`.
- Everything related to GML Decompilation is located in
  `src/libgm/gml/decompiler/`.
- Everything related to GML Compilation will be located in
  `src/libgm/gml/compiler/` (not started yet).
