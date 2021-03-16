# FVR_ENGINE

A data-driven text-based roleplaying game engine.

## TODO

Make foreground a sparse grid map and set to None if the glyph is ' '.
^ Also add this ' ' check for the outline grid.

Every Frame:

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
