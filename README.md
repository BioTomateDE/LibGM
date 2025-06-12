This crate is part of a bigger project: [AcornGM](https://github.com/BioTomateDE/AcornGM).

## Features
- **Deserialization** of GameMaker data files
- **Serialization** of GameMaker data files
- **Exporting** AcornGM mods by finding changes between two data files
- **Applying** AcornGM mods to a GameMaker game

## How to use as a dependency
- Add this line to your `Cargo.toml`:
   ```toml
   [dependencies]
   libgm = { git = "https://github.com/BioTomateDE/LibGM" }
   ```
- Now you can use these function exposed by LibGM:
  - `parse_data_file()`
  - `build_data_file()`
  - `export_mod()`
  - `apply_mod()`

