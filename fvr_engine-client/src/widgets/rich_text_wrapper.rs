//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::cell::{Ref, RefCell};
use std::cmp;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{Context, Result};

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;
use fvr_engine_parser::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::widgets::rich_text_writer::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const NEWLINE_CHARACTER: char = '\n';
const SPACE_CHARACTER: char = ' ';

//-------------------------------------------------------------------------------------------------
// Helper struct for storing current format state.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Default)]
struct FormatState {
    // Whether the tag string has changed and needs to be rebuilt.
    updated: bool,
    // The current tag string
    tag_string: RefCell<String>,
    // Optional layout tag value.
    layout: Option<String>,
    // Optional style tag value.
    style: Option<String>,
    // Optional size tag value.
    size: Option<String>,
    // Optional outlined tag value.
    outlined: Option<String>,
    // Optional foreground color tag value.
    foreground_color: Option<String>,
    // Optional background color tag value.
    background_color: Option<String>,
    // Optional outline color tag value.
    outline_color: Option<String>,
}

impl FormatState {
    //---------------------------------------------------------------------------------------------
    // Reset the format state's state.
    //---------------------------------------------------------------------------------------------
    pub fn clear(&mut self) {
        self.layout = None;
        self.style = None;
        self.size = None;
        self.outlined = None;
        self.foreground_color = None;
        self.background_color = None;
        self.outline_color = None;
    }

    //---------------------------------------------------------------------------------------------
    // Reset the format state from a parsed hint.
    //---------------------------------------------------------------------------------------------
    pub fn update_from_hint(&mut self, key: RichTextHintType, value: String) {
        match key {
            RichTextHintType::Layout => self.layout = Some(value),
            RichTextHintType::Style => self.style = Some(value),
            RichTextHintType::Size => self.size = Some(value),
            RichTextHintType::Outlined => self.outlined = Some(value),
            RichTextHintType::ForegroundColor => self.foreground_color = Some(value),
            RichTextHintType::BackgroundColor => self.background_color = Some(value),
            RichTextHintType::OutlineColor => self.outline_color = Some(value),
        }

        self.updated = true;
    }

    //---------------------------------------------------------------------------------------------
    // Lazily update and return a ref to the tag string.
    //---------------------------------------------------------------------------------------------
    pub fn tag_string(&mut self) -> Ref<String> {
        if !self.updated {
            return self.tag_string.borrow();
        }

        // Update the tag string in a new scope so the mut borrow doesn't fight the return ref.
        {
            let mut tag_string = self.tag_string.borrow_mut();
            tag_string.clear();

            if let Some(ref layout) = self.layout {
                *tag_string += &format!("<l:{}>", layout);
            }
            if let Some(ref style) = self.style {
                *tag_string += &format!("<st:{}>", style);
            }
            if let Some(ref size) = self.size {
                *tag_string += &format!("<si:{}>", size);
            }
            if let Some(ref outlined) = self.outlined {
                *tag_string += &format!("<o:{}>", outlined);
            }
            if let Some(ref foreground_color) = self.foreground_color {
                *tag_string += &format!("<fc:{}>", foreground_color);
            }
            if let Some(ref background_color) = self.background_color {
                *tag_string += &format!("<bc:{}>", background_color);
            }
            if let Some(ref outline_color) = self.outline_color {
                *tag_string += &format!("<bc:{}>", outline_color);
            }
        }

        self.updated = false;
        self.tag_string.borrow()
    }
}

//-------------------------------------------------------------------------------------------------
// RichTextWrapper parses and wraps and allows for drawing an expandable rich text string.
//-------------------------------------------------------------------------------------------------
#[derive(Default)]
pub struct RichTextWrapper {
    // Origin of the rich text wrapper.
    origin: (u32, u32),
    // Dimensions of the visible area.
    dimensions: (u32, u32),
    // Maximum number of wrapped lines.
    max_lines: u32,
    // Current # of lines in the rich text.
    total_lines: u32,
    // Current # of lines above the visible area.
    lines_up: u32,
    // Current # of lines below the visible area.
    lines_down: u32,
    // Cached format state to append at the beginning of lines.
    format_state: FormatState,
    // The wrapped rich text.
    wrapped_text: String,
    // Whether to prepend a space to the next appended text.
    prepend_space: bool,
    // Vec of newline indices in the rich text.
    newline_indices: Vec<usize>,
    // Current length of the last line.
    last_line_length: usize,
    // Start index of the visible area (in the rich text).
    visible_start: usize,
    // End index of the visible area (in the rich text).
    visible_end: usize,
    // Index of the newline at the beginning of the current visible area.
    current_line: usize,
}

impl RichTextWrapper {
    //---------------------------------------------------------------------------------------------
    // Creates a new rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: (u32, u32), dimensions: (u32, u32), max_lines: u32) -> Self {
        // Push a newline index for the beginning of the wrapped text.
        let newline_indices = vec![0];

        Self { origin, dimensions, max_lines, newline_indices, ..Default::default() }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the origin of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn origin(&self) -> (u32, u32) {
        self.origin
    }

    //---------------------------------------------------------------------------------------------
    // Returns the width of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

    //---------------------------------------------------------------------------------------------
    // Returns the height of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn height(&self) -> u32 {
        self.dimensions.1
    }

    //---------------------------------------------------------------------------------------------
    // Returns the dimensions of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    //---------------------------------------------------------------------------------------------
    // Returns the max lines of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn max_lines(&self) -> u32 {
        self.max_lines
    }

    //---------------------------------------------------------------------------------------------
    // Returns the total lines of the rich text wrapper's text.
    //---------------------------------------------------------------------------------------------
    pub fn total_lines(&self) -> u32 {
        self.total_lines
    }

    //---------------------------------------------------------------------------------------------
    // Returns the # of lines above the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn lines_up(&self) -> u32 {
        self.lines_up
    }

    //---------------------------------------------------------------------------------------------
    // Returns the # of lines below the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn lines_down(&self) -> u32 {
        self.lines_down
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether there are any lines above the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_lines_up(&self) -> bool {
        self.lines_up > 0
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether there are any lines below the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_lines_down(&self) -> bool {
        self.lines_down > 0
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether the content is longer than the visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_overflow(&self) -> bool {
        self.total_lines > self.height()
    }

    //---------------------------------------------------------------------------------------------
    // When handling hints we want to...
    // 1. Create the inline tag string to append to the wrapped text.
    // 2. Update the format state.
    // 3. Append the inline tag string to the wrapped text.
    //---------------------------------------------------------------------------------------------
    fn handle_hint(&mut self, key: RichTextHintType, value: String) {
        // Generate the inline format tag.
        let inline_tag = format!("<{}:{}>", key.to_key_value(), &value);

        // Update the format state.
        self.format_state.update_from_hint(key, value);

        // Append the inline format tag.
        self.wrapped_text.push_str(&inline_tag);
    }

    //---------------------------------------------------------------------------------------------
    // When handling newlines we want to...
    // 1. Append a newline to the wrapped text.
    // 2. Add a new newline descriptor for the current newline and tag string length.
    // 3. Append the current format state tag string.
    // 4. Reset the last line length.
    //---------------------------------------------------------------------------------------------
    fn handle_newline(&mut self) {
        // If the last character in the wrapped text is an empty space, remove it.
        if let Some(last_char) = self.wrapped_text.chars().rev().next() {
            if last_char == SPACE_CHARACTER {
                self.wrapped_text.pop();
            }
        }

        // Append a newline.
        self.wrapped_text.push(NEWLINE_CHARACTER);

        // Update the vec of newline descriptors.
        self.newline_indices.push(self.wrapped_text.chars().count());

        // Append the current format tag string.
        self.wrapped_text.push_str(&self.format_state.tag_string());

        // Reset the last line length.
        self.last_line_length = 0;
    }

    //---------------------------------------------------------------------------------------------
    // When handling text we want to...
    // 1. Split the text on whitespace and collect the resulting strings into a vec.
    // 2. Handle appending each word. Except for the last word, also append a whitespace.
    //---------------------------------------------------------------------------------------------
    fn handle_text(&mut self, text: String) {
        debug_assert!(!text.is_empty(), "Parsed an empty text value.");

        // Collect words into a vec because we need the length.
        let words: Vec<_> = text.split_whitespace().collect();
        let words_len = words.len();

        // If text begins with whitespace, or a word was appended last, add a space.
        if self.prepend_space {
            self.handle_word(" ", true);
        } else if let Some(first_char) = text.chars().next() {
            if first_char == SPACE_CHARACTER {
                self.handle_word(" ", true);
            }
        }

        // Handle appending each word and insert a single whitespace between (but not trailing).
        for (i, word) in words.into_iter().enumerate() {
            self.handle_word(word, false);

            if i < words_len - 1 {
                self.handle_word(" ", true);
            }
        }

        // If the text ends with whitespace, add a space.
        if let Some(last_char) = text.chars().last() {
            if last_char == SPACE_CHARACTER {
                self.handle_word(" ", true);
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // When handling words we want to...
    // 1. Check if the wrapped text has content and the last parsed value was not a tag and the
    //    last char was not special.
    //   a. If so, handle adding a space.
    // 2. Check if appending the word will wrap to a new line.
    //   a. If so, handle adding a newline.
    // 3. Append the word.
    // 4. Update the current line length.
    // 5. Set that spaces can be added (already true unless this was the first word) and the that
    //    the last value was not a tag.
    //---------------------------------------------------------------------------------------------
    fn handle_word(&mut self, word: &str, is_space: bool) {
        debug_assert!(!word.is_empty(), "Parsed an empty word.");

        // If there is not enough room to append the word on this line, break to the next line.
        if self.last_line_length + word.chars().count() > self.width() as usize {
            // Do not append spaces that would cause a line break.
            if is_space {
                return;
            }

            self.handle_newline();
        }

        // Append the word and update the last line length.
        self.wrapped_text.push_str(word);
        self.last_line_length += word.chars().count();
    }

    //---------------------------------------------------------------------------------------------
    // Refresh properties related to visible lines.
    // (should be called whenever the current line index or the wrapped text changes)
    //---------------------------------------------------------------------------------------------
    fn refresh_visible_area_metrics(&mut self) {
        // Total lines is the # of newlines.
        self.total_lines = self.newline_indices.len() as u32;

        // Lines up is always equal to the current newline index.
        self.lines_up = self.current_line as u32;

        // Lines down is the difference between total lines and the last visible line.
        self.lines_down = cmp::max(
            self.total_lines as i32 - (self.current_line as i32 + self.height() as i32),
            0,
        ) as u32;

        // Visible start is the newline index of the current line.
        self.visible_start = self.newline_indices[self.current_line];

        // Depending on whether there is room to fill the entire height of the text wrapper, set
        // the visible end index.
        if self.total_lines > self.height() {
            if self.current_line as u32 + self.height() >= self.total_lines {
                // The entire remainder of the wrapped text is visible.
                self.visible_end = self.wrapped_text.chars().count();
            } else {
                // The remainder of the wrapped text must be cut off after height.
                self.visible_end =
                    self.newline_indices[self.current_line + self.height() as usize] - 1;
            }
        } else {
            // The entire remainder of the wrapped text is visible.
            self.visible_end = self.wrapped_text.chars().count();
        }
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for truncating wrapped text.
    //---------------------------------------------------------------------------------------------
    fn truncate_text(&mut self) {
        if self.newline_indices.len() > self.max_lines as usize {
            // Find the new starting newline index.
            let index = self.newline_indices.len() - self.max_lines as usize;
            let start = self.newline_indices[index];

            // Truncate the newline indices;
            unsafe {
                let src = self.newline_indices.as_ptr().add(index);
                let dst = self.newline_indices.as_mut_ptr();
                std::ptr::copy(src, dst, self.max_lines as usize);
            }

            self.newline_indices.truncate(self.max_lines as usize);

            for n in self.newline_indices.iter_mut() {
                *n -= start;
            }

            // Truncate the wrapped text.
            let size = self.wrapped_text.len() - start;

            unsafe {
                let src = self.wrapped_text.as_ptr().add(start);
                let dst = self.wrapped_text.as_mut_ptr();
                std::ptr::copy(src, dst, size);
            }

            self.wrapped_text.truncate(size);
        }
    }

    //---------------------------------------------------------------------------------------------
    // Append rich text to the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn append(&mut self, text: &str) -> Result<()> {
        // Parse the rich text.
        let parsed_values = parse_rich_text(text).context("Failed to parse rich text string.")?;

        // Iterate over and handle each of the parsed values.
        for value in parsed_values.into_iter() {
            match value {
                RichTextValue::FormatHint { key, value } => {
                    self.handle_hint(key, value);
                    self.prepend_space = false;
                }
                RichTextValue::Newline => {
                    self.handle_newline();
                    self.prepend_space = false;
                }
                RichTextValue::Text(text) => {
                    self.handle_text(text);
                    self.prepend_space = true;
                }
            }
        }

        // Ensure the wrapped text is not longer than the max lines.
        self.truncate_text();

        // Always update visible area metrics.
        self.refresh_visible_area_metrics();

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area up by a # of lines.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_up(&mut self, lines: u32) {
        // Decrement the current line index, stopping at 0.
        self.current_line = cmp::max(self.current_line as i32 - lines as i32, 0) as usize;

        // Always update visible area metrics.
        self.refresh_visible_area_metrics();
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area down by a # of lines.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_down(&mut self, lines: u32) {
        // Only scroll down if there is text that might not be visible
        if self.has_overflow() {
            // Increment the current line index, stopping at the bottom of the visible area.
            self.current_line = cmp::min(
                self.current_line + lines as usize,
                (self.total_lines - self.height()) as usize,
            );

            // Always update visible area metrics.
            self.refresh_visible_area_metrics();
        }
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area to the top.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_to_top(&mut self) {
        self.current_line = 0;

        // Always update visible area metrics.
        self.refresh_visible_area_metrics();
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area to the bottom.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_to_bottom(&mut self) {
        // Only scroll if there is text that might not be visible.
        if self.has_overflow() {
            self.current_line = (self.total_lines - self.height()) as usize;

            // Always update visible area metrics.
            self.refresh_visible_area_metrics();
        }
    }

    //---------------------------------------------------------------------------------------------
    // Clear the contents of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn clear(&mut self) {
        self.current_line = 0;
        self.wrapped_text.clear();
        self.format_state.clear();
        self.newline_indices.clear();
        self.newline_indices.push(0);
    }

    //---------------------------------------------------------------------------------------------
    // Draws the rich text wrapper at the origin point.
    //---------------------------------------------------------------------------------------------
    pub fn draw<M>(&self, map: &mut M) -> Result<()>
    where
        M: Map2d<Tile>,
    {
        // Return if there is no text to draw.
        if self.total_lines < 1 || self.visible_end - self.visible_start < 1 {
            return Ok(());
        }

        // Clear the foreground glyph of the covered area.
        for x in self.origin.0..(self.origin.0 + self.width()) {
            for y in self.origin.1..(self.origin.1 + self.height()) {
                map.get_xy_mut((x, y)).glyph = SPACE_CHARACTER;
            }
        }

        // Create a slice of visible rich text.
        let visible_slice = &self.wrapped_text[self.visible_start..self.visible_end];

        // Draw the wrapped rich text.
        RichTextWriter::write(map, self.origin, visible_slice)?;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Clears the background and draws the rich text wrapper at the origin point.
    //---------------------------------------------------------------------------------------------
    pub fn draw_clear<M>(&self, map: &mut M) -> Result<()>
    where
        M: Map2d<Tile>,
    {
        // Clear the foreground glyph of the covered area.
        for x in self.origin.0..(self.origin.0 + self.width()) {
            for y in self.origin.1..(self.origin.1 + self.height()) {
                map.get_xy_mut((x, y)).glyph = SPACE_CHARACTER;
            }
        }

        self.draw(map)
    }
}
