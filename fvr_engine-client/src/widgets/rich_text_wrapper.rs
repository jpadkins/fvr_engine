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
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::widgets::rich_text_writer::*;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;
use fvr_engine_parser::prelude::*;

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
    // Width of the visible area.
    width: u32,
    // Height of the visible area.
    height: u32,
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
    pub fn new(width: u32, height: u32) -> Self {
        // Push a newline index for the beginning of the wrapped text.
        let newline_indices = vec![0];

        Self { width, height, newline_indices, ..Default::default() }
    }

    //---------------------------------------------------------------------------------------------
    // Return the width of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn width(&self) -> u32 {
        self.width
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the rich text wrapper.
    //---------------------------------------------------------------------------------------------
    pub fn height(&self) -> u32 {
        self.height
    }

    //---------------------------------------------------------------------------------------------
    // Return the total lines of the rich text wrapper's text.
    //---------------------------------------------------------------------------------------------
    pub fn total_lines(&self) -> u32 {
        self.total_lines
    }

    //---------------------------------------------------------------------------------------------
    // Return the # of lines above the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn lines_up(&self) -> u32 {
        self.lines_up
    }

    //---------------------------------------------------------------------------------------------
    // Return the # of lines below the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn lines_down(&self) -> u32 {
        self.lines_down
    }

    //---------------------------------------------------------------------------------------------
    // Return whether there are any lines above the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_lines_up(&self) -> bool {
        self.lines_up > 0
    }

    //---------------------------------------------------------------------------------------------
    // Return whether there are any lines below the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_lines_down(&self) -> bool {
        self.lines_down > 0
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
        self.newline_indices.push(self.wrapped_text.len());

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
        debug_assert!(text.is_empty() == false, "Parsed an empty text value.");

        // Collect words into a vec because we need the length.
        let words: Vec<_> = text.split_whitespace().collect();
        let words_len = words.len();

        // Handle appending each word and insert a single whitespace between (but not trailing).
        for (i, word) in words.into_iter().enumerate() {
            self.handle_word(word);

            if i < words_len - 1 {
                self.handle_word(" ");
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
    fn handle_word(&mut self, word: &str) {
        debug_assert!(word.is_empty() == false, "Parsed an empty word.");

        // If there is not enough room to append the word on this line, break to the next line.
        if self.last_line_length + word.len() >= self.width as usize {
            self.handle_newline();
        }

        // Append the word and update the last line length.
        self.wrapped_text.push_str(word);
        self.last_line_length += word.len();
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
        self.lines_down =
            cmp::max(self.total_lines as i32 - (self.current_line as i32 + self.height as i32), 0)
                as u32;

        // Visible start is the newline index of the current line.
        self.visible_start = self.newline_indices[self.current_line];

        // Depending on whether there is room to fill the entire height of the text wrapper, set
        // the visible end index.
        if self.total_lines > self.height {
            if self.current_line as u32 + self.height >= self.total_lines {
                // The entire remainder of the wrapped text is visible.
                self.visible_end = self.wrapped_text.len();
            } else {
                // The remainder of the wrapped text must be cut off after height.
                self.visible_end =
                    self.newline_indices[self.current_line + self.height as usize] - 1;
            }
        } else {
            // The entire remainder of the wrapped text is visible.
            self.visible_end = self.wrapped_text.len();
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
                RichTextValue::FormatHint { key, value } => self.handle_hint(key, value),
                RichTextValue::Newline => self.handle_newline(),
                RichTextValue::Text(text) => self.handle_text(text),
            }
        }

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
        if self.total_lines > self.height {
            // Increment the current line index, stopping at the bottom of the visible area.
            self.current_line = cmp::min(
                self.current_line + lines as usize,
                (self.total_lines - self.height) as usize,
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
        if self.total_lines > self.height {
            self.current_line = (self.total_lines - self.height) as usize;

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
    // Draws the rich text wrapper at an origin point.
    //---------------------------------------------------------------------------------------------
    pub fn draw<M>(&self, map: &mut M, xy: (u32, u32)) -> Result<()>
    where
        M: Map2d<Tile>,
    {
        // Return if there is no text to draw.
        if self.total_lines < 1 || self.visible_end - self.visible_start < 1 {
            return Ok(());
        }

        // Create a slice of visible rich text.
        let visible_slice = &self.wrapped_text[self.visible_start..self.visible_end];

        // Draw the wrapped rich text.
        RichTextWriter::write(map, xy, visible_slice)?;

        Ok(())
    }
}
