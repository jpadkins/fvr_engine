```
  ________ ___      ___ ________   _______   ________   ________  ___  ________   _______
 |\  _____\\  \    /  /|\   __  \ |\  ___ \ |\   ___  \|\   ____\|\  \|\   ___  \|\  ___ \
  \ \  \__/\ \  \  /  / | \  \|\  \\ \   __/|\ \  \\ \  \ \  \___|\ \  \ \  \\ \  \ \   __/|
   \ \   __\\ \  \/  / / \ \   _  _\\ \  \_|/_\ \  \\ \  \ \  \  __\ \  \ \  \\ \  \ \  \_|/__
    \ \  \_| \ \    / /   \ \  \\  \|\ \  \_|\ \ \  \\ \  \ \  \|\  \ \  \ \  \\ \  \ \  \_|\ \
     \ \__\   \ \__/ /     \ \__\\ _\ \ \_______\ \__\\ \__\ \_______\ \__\ \__\\ \__\ \_______\
      \|__|    \|__|/       \|__|\|__| \|_______|\|__| \|__|\|_______|\|__|\|__| \|__|\|_______|
```

A Rusty Re-Write of the _CabinFever_ game engine.

A data-driven text-based roleplaying game engine.

![renderer v2 example](screenshots/renderer_v2.png)

## TODO

- implement everything from C# engine.
  - [x] faux terminal rendering
  - [x] rich text parsing and drawing
  - [x] user input & derivative actions handling
  - [x] input repeater
  - [x] scenes:
    - [x] initial
    - [x] main menu
    - [x] demo (explore)
    - [ ] credits
    - [ ] help
    - [x] fade in/out animations
    - [x] transition animations
  - [ ] tile cache deserialized
  - [ ] cell cache deserialized
  - [ ] color palette deserialized
  - [ ] dynamic prefabs
  - [x] shadowcasting fov
  - [x] geometrical types
  - [ ] line drawing
  - [ ] modal popups
  - [x] scrolling dialogue windows

- implement "huge" characters
  - perhaps also per-tile implement animations that "live" in the client?
    - shake
    - rotate
    - wobble back and forth
    - grow and shrink

- implement static/dynamic color palettes
- implement ability to switch between vsync and custom fps in client.

- clean up server crate
- generalize scratch scene into base version of explore scene

## CRATES

### fvr_engine
The main FVR_ENGINE crate. Runs a game from config files.

Includes a "scene stack" for managing game scenes.

### fvr_engine-atlas

Utility for generating atlas textures for codepage 437 from TTF fonts.
Ensures that all codepoints are covered by filling in any missing entries with DejaVuSansMono.

TODO: Write more detailed tutorial

Basic steps:

  - generate 64pt regular, italic, bold, bold-italic & outline versions for each from bmfont
    - use glyphs.txt to choose codepoints
  - rename to remove page number from filenames
  - open each outlined texture in gimp and do the following:
    - pick `color to alpha` to remove white
    - pick `colorize` to change black outlined back to white
  - stick in directory under fonts/
  - run with `cargo run -p fvr_engine-atlas -- run` while in root dir of project
  - output goes to resources/fonts/
  - finally, create SDF versions of the textures. See the bash script in resources/fonts.

### fvr_engine-client
Lib for handling the game window, user input, and drawing to the faux terminal.

### fvr_engine-core
Lib containing commonly shared types.

- traits for map2d
- tile and related structs
- glyph/font metrics for deserialization
- timer
- translate map for viewing map2d with translation
- grid map for exposing map2d backed by vec
- cp437 helpers

### fvr_engine-parser
Lib containing text parsers, including:
- a rich text parser for parsing text that contains inline format hints

### fvr_engine-server
Lib for handling the game's business logic, such as characters, items, zones, combat, etc...

## TASKS

- continue adapting C# implementation.
- refactor frame to use rect?
- move transitions to client?
- implement shake effect
- determine if priority values in flee map should be negated like astar.
  x I don't think this is necessary anymore with the changes to fleemap, but double check.

## CONVENTIONS

- Frames work best with _odd_ dimensions.
- Managing cursor should _always_ be done at the root scene level.
- includes order:
  - "STD Includes."
  - "Extern crate includes."
  - "Workspace includes."
  - "Local includes."
  - "Constants."
  - "Statics."
