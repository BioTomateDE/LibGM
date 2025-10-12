> This crate is part of a bigger project: [AcornGM](https://github.com/BioTomateDE/AcornGM)

A Rust port of UndertaleModLib. Originally intended to provide a modding system for GameMaker data files.

However for now, the focus has shifted to just plagarising Underanalyzer.

## Features
- **Deserialization** of GameMaker data files
- **Serialization** of GameMaker data files
- **Exporting** AcornGM mods by finding changes between two data files
- **Applying** AcornGM mods to a GameMaker game

## How to use as a dependency
Add this line in the `[dependencies]` section of your `Cargo.toml` file:
```toml
libgm = { git = "https://github.com/BioTomateDE/LibGM" }
```
Now you can use these function exposed by LibGM:
- `parse_data_file()`
- `build_data_file()`
- `export_mod()` (currently unavailable)
- `apply_mod()`  (not yet implemented)
- `decompile_to_ast()` (not finished)

## Credits
Huge thanks to the Underminers Team; without [UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool) this project would've been impossible. I also want to thank the people in the Underminers Discord who helped me along the way, especially [@colinator27](https://github.com/colinator27).

## Licence
This project is licenced under [GPL v3](https://www.gnu.org/licenses/gpl-3.0.en.html) (free).

A separate commercial licence is also available (contact [latuskati+licence@gmail.com](mailto:latuskati+licence@gmail.com)).
