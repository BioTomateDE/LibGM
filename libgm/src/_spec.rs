// SPDX-License-Identifier: GPL-3.0-only

//! Some yapping about the data file format.
//!
//! ## History
//! GameMaker is a Game Engine originally created by Mark Overmars in 1999.
//! It was called "Animo" in its very early days, before being renamed to "Game Maker".
//! Back then, this game engine was actually versioned sensibly: 1.1, 1.4, 4.3b, 6.1, etc.
//! The game assets were directly embedded in the EXE; there was no data file.
//! Since this format is completely different, LibGM does not support it.
//! In the final version of 8.1 (April 2011), it was renamed to "GameMaker" (without the space).
//! Then, its successor "GameMaker: Studio" was released in May 2012.
//! Here, they introduced a new binary format for storing game assets in the new data files.
//! It was versioned 1.0 to 1.4, with internal build numbers up to 1784 or something (and 9999).
//! In 2015, GameMaker was aquired by the company YoYo Games.
//! In 2017, GameMaker Studio 2 was released (yes, they removed the colon from the name).
//! I'm too lazy to finish this section.
//!
//! ## Terminology
//! LibGM handles "GameMaker data files". This is what I call them.
//!
//! Data files are also known as a "WAD files" which is a term from
//! the game Doom and stands for "Where's All the Data?".
//! See more on [Wikipedia](https://en.wikipedia.org/wiki/Doom_modding).
//!
//! Another (more general) term for them is "IFF files", which stands for Interchange File Format.
//! IFF describes the chunk based format of data files:
//! there are 4-character "type IDs" (aka. chunk names, FourCC or OSType) followed
//! by a 32-bit integer denoting the length of the chunk. More on that later.
//!
//! The abbreviations YY and YYG refer to YoYo Games.
//!
//! GM stands for GameMaker. I also use it for versioning (e.g. GM 2023.2).
//! GMS stands for GameMaker Studio (also written as GM:S before Studio 2).
//!
//! I also use WAD for versioning (e.g. WAD 15 means WAD/bytecode version 15).
//! WAD version and bytecode version mean the same thing (the latter is slightly outdated/incorrect UTMT terminology).
//!
//! GMAC is short for GMAssetCompiler.
//! This is a program found in the GameMaker IDE directory that generates data files.
//! It's written in C#, so it's easy to decompile. Lots of datamining was done on it.
//!
//! UTMT and UMT stand for UndertaleModTool, which is the most popular and complete tool for unpacking/modding GameMaker data files.
//! LibGM ~~steals~~ borrows lots of code from it (thanks bro).
//! UTMT can refer to the GUI application or to the entire project (which includes UTML).
//! UTML stands for UndertaleModLib, the underlying data data file (de)serialization library of UTMT.
//! UA stands for Underanalyzer, which is a separate repo developed (mainly) by colinator27 (the goat).
//! It is a decompiler and compiler for GML, used by UndertaleModTool.
//!
//! GameMaker has its own programming language called "GML" (GameMaker Language).
//! This language is (usually\*) compiled to GML bytecode which
//! is then run in a VM; making it portable to every platform.
//!
//! ## Distinction between data file and runner
//!
//! A compiled GameMaker game consists of two main components:
//! * The data file
//! * The runner
//!
//! The data file (typically named `data` or `game`)
//! contains all the game's static resources in a binary format.
//! Common file extensions include `.win`, `.unx`, `.droid`, and `.ios`
//! for their respective platforms,
//! though YoYo Games sometimes uses `.win` for console targets as well.
//!
//! This data file stores virtually all game content, including:
//! * Textures and sprite data
//! * Audio files
//! * Rooms
//! * Game objects
//! * GML bytecode (unless YYC)
//! * Other game assets and configuration
//!
//! **This library (LibGM) provides tools to decode and modify these GameMaker data files.**
//!
//! The runner is the executable program that loads and runs the data file. It
//! handles:
//! * Input processing (keyboard, mouse, controllers)
//! * Video and audio output
//! * Game state management
//! * Real-time game execution
//!
//! The runner is a reusable\* executable that can run any
//! GameMaker game built for the same version.
//!
//! The data file contains the unique content and logic that defines your
//! specific game. When launched, the runner loads the data file and executes
//! the game contained within it.
//!
//! # VM vs YYC
//! You may have noticed some stars regarding bytecode while reading.
//! This is because YoYo Games has created `YYC` ([YoYo Compiler](https://manual.gamemaker.io/monthly/en/Settings/YoYo_Compiler.htm)).
//! When exporting their game, game developers can choose between regular VM output or YYC output.
//!
//! In VM mode, the GML source code is compiled to their own bytecode that
//! then gets executed by the runner's special GML Virtual Machine.
//! The VM bytecode is stored in the data file and is therefore
//! able to be decoded and modified by LibGM.
//!
//! In YYC, the GML source code is converted to C code
//! and then compiled by GCC (DOCME: not sure).
//! The main point is:
//! The data file doesn't store anything code related for YYC.
//! Instead, all the game's code is compiled to machine code in the specialized
//! runner. In this case, the runner does not execute bytecode in a VM but
//! instead executes compiled machine code directly embedded in its executable
//! file. Therefore, YYC runners are not portable to other games.
//!
//!
//! ## Data file format
//! The GameMaker data file format is binary.
//! (It only contains text for chunk names and in the `STRG` chunk.)
//!
//! The data file consists of "chunks", although the name "chunk" may be a bit
//! misleading, as these chunks do not have a fixed size and cannot appear multiple times.
//! You could think of them as sections if you like.
//! (I will continue to call them chunks because that is the offical name  *and* best-known community name for them.)
//! You can see these chunk names in the console output in GameMaker Studio when building your game.
//!
//! Every chunk has a 4 character name that indicates what it stores.
//! For example: `SPRT` stores sprites, `ROOM` stores rooms, `OBJT` stores game objects, etc.
//! These chunk names are hardcoded (the game dev does not choose them).
//! New ones are only added when YoYo Games adds a major feature like particles.
//!
//! Every chunk only exists once. There is only one `ROOM` chunk
//! that stores **all** data related to GameMaker rooms.
//!
//! The order of these chunks matters, because the runner loads them sequentially.
//! Wrong initialization order can result in blackscreens, segfaults, errors or missing audio.
//! Just don't touch `ChunkOrder` and you should be good lol.
//!
//! Now, onto the actual structure of a data file.
//! Every data file begins with the 4 letters `FORM`.
//! > Note: On Big endian targets, it is `MROF` instead, as chunk names are reversed there.
//! > Do not worry about this though, big endian is very legacy
//! > and is not relevant for modern target platforms (only used on PS3 or something).
//!
//! After the 4 byte `FORM` string, you are met with the (remaining) data length.
//! This is equivalent to the size of the data file, minus 8 (the FORM header is excluded).
//! This data length is specified as a 32-bit integer (`u32`).
//!
//! Then, you will see the first chunk.
//! > Note: Some GameMaker unpacking software (like UndertaleModLib) also see FORM as a chunk.
//! > I do not do this, as this is the only time chunks have any sort of hierarchy
//! > and it would unnecessarily make things more complicated.
//!
//! Chunks are structured similar to FORM:
//! * Chunk name [4 bytes]
//! * Chunk length (excluding this header) [4 bytes]
//! * Chunk data [n bytes]
//! * Potentially chunk padding (nullbytes), depending on the platform and
//!   version.
//!
//! Most chunks are of "pointer list type" (UndertaleModLib calls them `UndertaleListChunk`).
//! To understand what that means, we first need to look at list structures in data files.
//!
//! ## Lists
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
//! In other words, they store their element count, followed by the element
//! data. Simple indeed. (DOCME: are the element sizes always the same?)
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
//! > Note: Null/Zeroed pointers are always invalid, as they would point to the
//! > FORM string.
//! > YoYo Games has recently implemented a system to remove unused assets,
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
//! The runner would have to deserialize all elements
//! sequentially just to get to the one it actually wants to read.
//! Depending on how those elements are typically used, this can be very
//! inefficient, which is why YoYo Games includes pointers to the element in
//! pointer lists.
//!
//!
//! ## Chunk types
//! Now that you know what pointer lists are, I can explain different "chunk
//! types". Most chunks store a collection of elements and are therefore a giant
//! pointer list. For example: `ROOM` stores multiple rooms, `FONT` stores
//! multiple fonts, etc. There are a few exceptions, though.
//!
//! The `FEAT` chunk ("feature flags") consists of only a simple
//! list of string references (which are pointers anyway).
//!
//! The `GEN8` and `OPTN` contain general info and options regarding the data
//! file / game. They are called `UndertaleSingleChunk` in UndertaleModLib.
//! They do not contain a list. Instead, they are the
//! element itself and contain the fields directly.
//!
//!
//! # Epilogue
//! i wrote this in one session like one year ago.
//! it's not the best doc of all time.
//! feel free to make a pr if you want to improve this yap or have more to say :)
