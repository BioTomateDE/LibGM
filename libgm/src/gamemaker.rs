//! Everything related to parsing and building of GameMaker data files.
//!
//! GameMaker is a Game Engine created by YoYoGames.
//! You will see the abbreviations "GM" and "YY" a lot.
//!
//! GameMaker has its own programming language called "GML".
//! This language is (usually\*) compiled to GML bytecode which
//! is then run in a VM; making it portable to every platform.
//!
//! # Distinction between data file and runner
//!
//! A compiled GameMaker game consists of two main components:
//! * The data file
//! * The runner
//!
//! The data file (typically named `data.xxx` or `game.xxx`)
//! contains all the game's static resources in a binary format.
//! Common file extensions include `.win`, `.unx`, `.droid`, and `.ios`
//! for their respective platforms,
//! though YoYoGames sometimes uses `.win` for console targets as well.
//!
//! This data file stores virtually all game content, including:
//! * Textures and sprite data
//! * Audio files
//! * Rooms
//! * Game objects
//! * GML bytecode (unless YYC)
//! * Other game assets and configuration
//!
//! **This library (LibGM) provides tools to parse and modify these GameMaker data files.**
//!
//! The runner is the executable program that loads and runs the data file. It handles:
//! * Input processing (keyboard, mouse, controllers)
//! * Video and audio output
//! * Game state management
//! * Real-time game execution
//!
//! The runner is a reusable\* executable that can run any
//! GameMaker game built for the same version.
//!
//! The data file contains the unique content and logic that defines your specific game.
//! When launched, the runner loads the data file and executes
//! the game contained within it.
//!
//! # VM vs YYC
//! You may have noticed some stars regarding bytecode while reading.
//! This is because YoYoGames has created `YYC` (TODO: abbreviation meaning missing).
//! When exporting their game, game developers can choose between regular VM output or YYC output.
//!
//! In VM mode, the GML source code is compiled to their own bytecode that
//! then gets executed by the runner's special GML Virtual Machine.
//! The VM bytecode is stored in the data file and is therefore
//! able to be parsed, decompiled and modified by LibGM.
//!
//! In YYC, the GML source code is converted to C code
//! and then compiled by GCC (TODO: not sure).
//! The main point is:
//! The data file doesn't store anything code related for YYC.
//! Instead, all the game's code is compiled to machine code in the specialized runner.
//! In this case, the runner does not execute bytecode in a VM but instead executes
//! compiled machine code directly embedded in its executable file.
//! Therefore, YYC runners are not portable to other games.
//!
//!
//! ## Data file format
//! As I already said, the GameMaker data file format is binary.
//! (It only contains text for chunk names and in the `STRG` chunk.)
//!
//! The data file consists of "chunks", although the name "chunk" may be a bit misleading,
//! as these chunks do not have a fixed size.

//! > Note: These are the same chunks you see in the console
//! > output in GameMaker Studio when building your game.
//!
//! Every chunk has a 4 character name that indicates what it stores.
//! (For Example: `SPRT` stores sprites, `ROOM` stores rooms, `OBJT` stores game objects, etc.)
//! These chunk names are hardcoded (the game dev does not choose them).
//! New ones are only added when YoYoGames adds a major feature like particles.
//!
//! Every chunk only exists once. There is only one `ROOM` chunk
//! that stores **all** data related to GameMaker rooms.
//!
//! The order of these chunks does not matter.
//! > TODO: Possible exception: `GEN8` needs to be first, at least for `UndertaleModLib`?
//!
//! Now, onto the actual structure of a data file.
//! Every data file begins with the 4 letters `FORM`.
//! > Note: On Big endian targets, it is `MROF` instead, as chunk names are reversed there.
//! > Do not worry about this though, big endian is very legacy
//! > and is not relevant for modern target platforms.
//!
//! After the 4 byte `FORM` string, you are met with the (remaining) data length.
//! This equivalent to the size of the data file minus 8 (the FORM header is excluded).
//! This data length is specified as a 32-bit integer (`u32`).
//!
//! Then, you will see the first chunk.
//! > Note: Some GameMaker datamining software (like UndertaleModLib) also see FORM as a chunk.
//! > I do not do this, as this is the only time chunks have any sort of hierarchy
//! > and it would unnecessarily make things more complicated.
//!
//! Chunks are structured similar to FORM:
//! * Chunk name [4 bytes]
//! * Chunk length (excluding this header) [4 bytes]
//! * Chunk data [n bytes]
//! * Potentially chunk padding (nullbytes), depending on the platform and version.
//!
//! Most chunks are of "pointer list type"; UndertaleModLib calls them `UndertaleListChunk`.
//! To understand what that means, we first need to look at list structures in data files.
//!
//! # Lists
//!
//! There are two common list types:
//! * Simple lists
//! * Pointer lists
//!
//! Simple lists have the following structure:
//! * Element count [4 bytes]
//! * Element #1 [n bytes]
//! * Element #2 [n bytes]
//! * ...
//! * Element #count-1 [n bytes]
//! * Element #count [n bytes]
//!
//! In other words, they store their element count, followed by the element data.
//! Simple indeed. (TODO: are the element sizes always the same?)
//!
//! Pointer lists have the following structure:
//! * Element count [4 bytes]
//! * Pointer to element #1 [4 bytes]
//! * Pointer to element #2 [4 bytes]
//! * ...
//! * Pointer to element #count-1 [4 bytes]
//! * Pointer to element #count [4 bytes]
//! * Element #1 [n bytes]
//! * Element #2 [n bytes]
//! * ...
//! * Element #count-1 [n bytes]
//! * Element #count [n bytes]
//!
//! In other words, they store their element count, a list of 32-bit pointers to
//! the corresponding element data and then their actual elements' data.
//!
//! A pointer is just a `u32` number. It specifies the absolute position where
//! something is located in the data file.
//! > Note: Null/Zeroed pointers are always invalid, as they would point to the FORM string.
//! > YoYoGames has recently implemented a system to remove unused assets,
//! > which nulls out pointers that point to unused elements.
//! > This is a pain in the ass for datamining libraries like mine,
//! > and LibGM still support them yet.
//!
//! *A bit of technical insight to pointer lists:*
//! You might be thinking how pointer lists store redundant data
//! and take up more space than simple lists.
//! This is true! They are indeed redundant.
//! However, some GameMaker elements may have a varying data length,
//! which makes simple pointer arithmetic impossible in simple lists.
//! The runner would have to parse all elements
//! sequentially just to get to the one it actually wants to read.
//! Depending on how those elements are typically used, this can be very inefficient,
//! which is why YoYoGames includes pointers to the element in pointer lists.
//!
//!
//! # Chunk types
//! Now that you know what pointer lists are, I can explain different "chunk types".
//! Most chunks store a collection of elements and are therefore a giant pointer list.
//! For example: `ROOM` stores multiple rooms, `FONT` stores multiple fonts, etc.
//! There are a few exceptions, though.
//!
//! The `FEAT` chunk ("feature flags") consists of only a simple
//! list of string references (which are pointers anyway).
//!
//! The `GEN8` and `OPTN` contain general info and options regarding the data file / game.
//! They are called `UndertaleSingleChunk` in UndertaleModLib.
//! They do not contain a list. Instead, they are the
//! element itself and contain the fields directly.
//!
//!

mod chunk;
mod version_detection;

pub mod data;
pub mod deserialize;
pub mod elements;
pub mod reference;
pub mod serialize;
pub mod version;
