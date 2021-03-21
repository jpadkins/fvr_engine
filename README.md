# FVR_ENGINE

A Rusty Re-Write of the _CabinFever_ game engine.

A data-driven text-based roleplaying game engine.

## TODO

- implement everything from C# engine.

## CRATES

### fvr_engine
The main FVR_ENGINE crate. Runs a game from config files.

### fvr_engine-core
Lib containing commonly shared types.

### fvr_engine-atlas_generator
Utility for generating atlas textures for codepage 437 from TTF fonts.
Ensures that all codepoints are covered by filling in any missing entries with DejaVuSansMono.

### fvr_engine-client
Lib for handling the game window and drawing to the faux terminal.

### fvr_engine-parser
Lib containing text parsers, including:
- a rich text parser for parsing text that contains inline format hints

## TASKS

- continue adapting C# implementation
