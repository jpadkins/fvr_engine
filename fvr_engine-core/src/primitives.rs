use sdl2::pixels::Color;

#[derive(Clone, Copy, Debug)]
pub struct TileColor(Color);

impl TileColor {
    pub const RED: TileColor = TileColor(Color::RED);
    pub const BLUE: TileColor = TileColor(Color::BLUE);
    pub const GREEN: TileColor = TileColor(Color::GREEN);
    pub const WHITE: TileColor = TileColor(Color::WHITE);
    pub const BLACK: TileColor = TileColor(Color::BLACK);
    pub const TRANSPARENT: TileColor = TileColor(Color::RGBA(255, 255, 255, 255));
}

#[derive(Clone, Copy, Debug)]
pub enum TileLayout {
    Center,
    Floor,
    Text,
    Exact((i32, i32)),
}

impl Default for TileLayout {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub glyph: char,
    pub layout: TileLayout,
    pub outlined: bool,
    pub foreground_color: TileColor,
    pub background_color: TileColor,
    pub outline_color: TileColor,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            glyph: '?',
            layout: Default::default(),
            outlined: false,
            foreground_color: TileColor::BLUE,
            background_color: TileColor::RED,
            outline_color: TileColor::TRANSPARENT,
        }
    }
}
