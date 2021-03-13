# FVR_ENGINE

A data-driven text-based roleplaying game engine.

## CRATES

### fvr_engine
The main FVR_ENGINE crate. Runs a game from config files.

### fvr_engine-core
Lib containing commonly shared types.

### fvr_engine-atlas_generator
Utility for generating atlas textures for codepage 437 from TTF fonts.
Ensures that all codepoints are covered by filling in any missing entries with DejaVuSansMono.

### fvr_engine-parser
Lib containing text parsers, including:
- a rich text parser for parsing text that contains inline format hints

### fvr_engine-renderer
Lib for handling the game window and drawing to the faux terminal.

## TASKS

- continue adapting C# implementation
