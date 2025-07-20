This crate is part of a bigger project: [AcornGM](https://github.com/BioTomateDE/AcornGM).

## Features
- **Deserialization** of GameMaker data files
- **Serialization** of GameMaker data files
- **Exporting** AcornGM mods by finding changes between two data files
- **Applying** AcornGM mods to a GameMaker game

## How to use as a dependency
- Add this line in the `[dependencies]` section of your `Cargo.toml` file:
   ```toml
   libgm = { git = "https://github.com/BioTomateDE/LibGM" }
   ```
- Now you can use these function exposed by LibGM:
  - `parse_data_file()`
  - `build_data_file()`
  - `export_mod()`
  - `apply_mod()` (todo)


## Credits
Huge thanks to the Underminers Team; without [UndertaleModLib](https://github.com/UnderminersTeam/UndertaleModTool) this project would've been impossible. I also want to thank the people in the Underminers Discord who helped me along the way, especially @colinator27.
