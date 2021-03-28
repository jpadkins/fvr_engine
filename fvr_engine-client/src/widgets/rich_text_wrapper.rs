// BEWARE YE WHO ENTER HERE!
// This code is ANCIENT and EVIL and GENERALLY VERY MESSY.

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
// Constants.
//-------------------------------------------------------------------------------------------------
const NEWLINE_CHARACTER: char = '\n';
const SPACE_CHARACTER: char = ' ';

//-------------------------------------------------------------------------------------------------
// Helper struct for storing extra info needed for newlines.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default)]
struct NewlineDescriptor {
    pub index: usize,
    pub offset: usize,
}

//-------------------------------------------------------------------------------------------------
// Helper struct for storing current format state.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Default)]
struct FormatState {
    updated: bool,
    tag_string: RefCell<String>,
    layout: Option<String>,
    style: Option<String>,
    size: Option<String>,
    outlined: Option<String>,
    foreground_color: Option<String>,
    background_color: Option<String>,
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

    pub fn tag_string_len(&self) -> usize {
        self.tag_string.borrow().len()
    }
}

//-------------------------------------------------------------------------------------------------
// RichTextWrapper parses and wraps and allows for drawing an expandable rich text string.
//-------------------------------------------------------------------------------------------------
#[derive(Default)]
pub struct RichTextWrapper {
    width: u32,
    height: u32,
    total_lines: u32,
    lines_up: u32,
    lines_down: u32,
    format_state: FormatState,
    wrapped_text: String,
    newlines: Vec<NewlineDescriptor>,
    last_line_length: usize,
    visible_start_index: usize,
    visible_end_index: usize,
    current_line_index: usize,
}

impl RichTextWrapper {
    pub fn new(width: u32, height: u32) -> Self {
        let newlines = vec![NewlineDescriptor { index: 0, offset: 0 }];

        Self { width, height, newlines, ..Default::default() }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn total_lines(&self) -> u32 {
        self.total_lines
    }

    pub fn lines_up(&self) -> u32 {
        self.lines_up
    }

    pub fn lines_down(&self) -> u32 {
        self.lines_down
    }

    pub fn has_lines_up(&self) -> bool {
        self.lines_up > 0
    }

    pub fn has_lines_down(&self) -> bool {
        self.lines_down > 0
    }

    // When handling hints we want to...
    // 1. Create the inline tag string to append to the wrapped text.
    // 2. Update the format state.
    // 3. Update the offset for the current line to reflect the added tag.
    // 4. Append the inline tag string to the wrapped text.
    fn handle_hint(&mut self, key: RichTextHintType, value: String) {
        // Generate the inline format tag.
        let inline_tag = format!("<{}:{}>", key.to_key_value(), &value);

        // Update the format state.
        self.format_state.update_from_hint(key, value);

        // Newlines will always contain at least one entry.
        self.newlines.last_mut().unwrap().offset += self.format_state.tag_string_len();

        // Append the inline format tag.
        self.wrapped_text.push_str(&inline_tag);
    }

    // When handling newlines we want to...
    // 1. Append a newline to the wrapped text.
    // 2. Add a new newline descriptor for the current newline and tag string length.
    // 3. Append the current format state tag string.
    // 4. Reset the last line length.
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
        self.newlines.push(NewlineDescriptor {
            index: self.wrapped_text.len(),
            offset: self.format_state.tag_string_len(),
        });

        // Append the current format tag string.
        self.wrapped_text.push_str(&self.format_state.tag_string());

        // Reset the last line length.
        self.last_line_length = 0;
    }

    fn handle_text(&mut self, text: String) {
        debug_assert!(text.is_empty() == false, "Parsed an empty text value.");

        // Collect words into a vec because we need the length.
        let words: Vec<_> = text.split_whitespace().collect();
        let words_len = words.len();

        // Handle appending each word and insert a single whitespace between (but not trailing).
        for (i, word) in words.into_iter().enumerate() {
            self.handle_word(word);

            if i < words_len - 1 {
                self.handle_char(SPACE_CHARACTER);
            }
        }
    }

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

    // When handling chars we want to...
    // 1. Check if appending the char will wrap to a new line.
    //   a. If so, handle adding a newline.
    // 2. Append the char.
    // 3. Update the current line length.
    // 4. Set that the last obj was not a tag.
    fn handle_char(&mut self, c: char) {
        // If there is not enough room to append the char on this line, break to the next line.
        if self.last_line_length + 1 >= self.width as usize && c != SPACE_CHARACTER {
            self.handle_newline();
        }

        // Append the char and update the last line length.
        self.wrapped_text.push(c);
        self.last_line_length += 1;
    }

    fn refresh_visible_area_metrics(&mut self) {
        // Update lines metrics.
        self.total_lines = self.newlines.len() as u32;
        self.lines_up = self.current_line_index as u32;

        // Lines down is the difference between total lines and the last visible line.
        self.lines_down = cmp::max(
            self.total_lines as i32 - (self.current_line_index + self.height as usize) as i32,
            0,
        ) as u32;

        // Visible start is the newline index of the current line.
        self.visible_start_index = self.newlines[self.current_line_index as usize].index;

        // Depending on whether there is room to fill the entire height of the text wrapper, set
        // the visible end index.
        if self.total_lines > self.height {
            let max_visible_index = self.current_line_index as u32 + self.height;

            if max_visible_index >= self.total_lines {
                // The entire remainder of the wrapped text is visible.
                self.visible_end_index = self.wrapped_text.len();
            } else {
                // The remainder of the wrapped text must be cut off after height.
                self.visible_end_index =
                    self.newlines[self.current_line_index + self.height as usize].index - 1;
            }
        } else {
            // The entire remainder of the wrapped text is visible.
            self.visible_end_index = self.wrapped_text.len();
        }
    }

    pub fn append(&mut self, text: &str) -> Result<()> {
        let parsed_values = parse_rich_text(text).context("Failed to parse rich text string.")?;

        for value in parsed_values.into_iter() {
            match value {
                RichTextValue::FormatHint { key, value } => self.handle_hint(key, value),
                RichTextValue::Newline => self.handle_newline(),
                RichTextValue::Text(text) => self.handle_text(text),
            }
        }

        self.refresh_visible_area_metrics();

        Ok(())
    }

    pub fn scroll_up(&mut self, lines: u32) {
        self.current_line_index = cmp::max(self.current_line_index - lines as usize, 0);
        self.refresh_visible_area_metrics();
    }

    pub fn scroll_down(&mut self, lines: u32) {
        if self.total_lines > self.height {
            self.current_line_index = cmp::min(
                self.current_line_index + lines as usize,
                (self.total_lines - self.height) as usize,
            );
            self.refresh_visible_area_metrics();
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.current_line_index = 0;
        self.refresh_visible_area_metrics();
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.total_lines > self.height {
            self.current_line_index = (self.total_lines - self.height) as usize;
            self.refresh_visible_area_metrics();
        }
    }

    pub fn draw(&self, xy: (u32, u32)) {
        if self.total_lines < 1 {
            return;
        }

        let visible_slice = &self.wrapped_text[self.visible_start_index..self.visible_end_index];

        println!("Drawing:\n{}", visible_slice);
    }
}
