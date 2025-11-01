# LibGM
A tool for unpacking, decompiling and modding 
GameMaker games such as Undertale or Deltarune.

This is effectively a Rust port of 
[UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool)
(specifically UndertaleModLib and [Underanalyzer](https://github.com/UnderminersTeam/Underanalyzer)).

# Benefits of this Rust port
- Runtime for parsing and building is ~8x faster than UndertaleModLib.
- Thorough documentation and clean code (half todo).
- Strict data integrity checks (can be disabled if unwanted).
- It's Rust, c'mon.

# Disadvantages / TODOs
- No GUI yet, only a Rust library.
- Null pointers are not yet supported.
- Decompiler and compiler not nearly done.

# How to use as a dependency
Add this line in the `[dependencies]` section of your `Cargo.toml` file:
```toml
libgm = { git = "https://github.com/BioTomateDE/LibGM" }
```

*This crate will also be added to [crates.io](https://https://crates.io/) when it is finished.*

Now you can use these function exposed by LibGM:
- `parse_data_file()`
- `build_data_file()`
- `decompile_to_ast()` (WIP)

# Credits
Huge thanks to the Underminers Team!
Without UndertaleModTool, this project would've been impossible.
I also want to thank the people in the Underminers Discord Server
who helped me along the way, especially [@colinator27](https://github.com/colinator27).

# Licencing
This project is licenced under the
[GNU Public Licence v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html)
(GPL-3).

# Contributing
All contributions are welcome!
Whether that's a pull request, a feature you would like to see added, a bug you found; 
just create an Issue/PR in this repo.
