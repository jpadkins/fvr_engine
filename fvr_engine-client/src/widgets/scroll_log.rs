//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::input_manager::*;
use crate::widgets::frame::*;
use crate::widgets::rich_text_wrapper::*;
use crate::widgets::scrollbar::*;

//-------------------------------------------------------------------------------------------------
// Enumerates the response codes when updating a scroll log.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollLogAction {
    // The scroll log was not interacted with.
    Noop,
    // The scroll log has focus (consumed user input).
    Focused,
    // The mouse is over an interactable area of the scroll log.
    Interactable,
}

//-------------------------------------------------------------------------------------------------
// ScrollLog manages a scrolling, wrapped rich-text log.
//-------------------------------------------------------------------------------------------------
pub struct ScrollLog {
    // The origin of the log.
    origin: (u32, u32),
    // The size of the log.
    dimensions: (u32, u32),
    // The frame around the scroll log.
    frame: Frame,
    // The scrollbar for the log.
    scrollbar: Scrollbar,
    // The wrapper for the log's contents.
    wrapper: RichTextWrapper,
    // Whether the scroll log needs to be redrawn.
    dirty: bool,
}

impl ScrollLog {
    //---------------------------------------------------------------------------------------------
    // Creates a new scroll log.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: (u32, u32), dimensions: (u32, u32), style: FrameStyle) -> Self {
        let frame = Frame::new(origin, (dimensions.0 - 2, dimensions.1 - 2), style);

        // Subtract from the height to nest the scrollbar within the frame.
        let scrollbar_origin = (origin.0 + dimensions.0 - 2, origin.1 + 1);
        let scrollbar = Scrollbar::new(scrollbar_origin, dimensions.1 - 2, 0);

        // Subtract from the dimensions to account for the frame and the scrollbar column.
        let wrapper_origin = (origin.0 + 1, origin.1 + 1);
        let wrapper = RichTextWrapper::new(wrapper_origin, (dimensions.0 - 3, dimensions.1 - 2));

        Self { origin, dimensions, frame, scrollbar, wrapper, dirty: true }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the origin of the scroll log.
    //---------------------------------------------------------------------------------------------
    pub fn origin(&self) -> (u32, u32) {
        self.origin
    }

    //---------------------------------------------------------------------------------------------
    // Returns the width of the scroll log.
    //---------------------------------------------------------------------------------------------
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

    //---------------------------------------------------------------------------------------------
    // Returns the height of the scroll log.
    //---------------------------------------------------------------------------------------------
    pub fn height(&self) -> u32 {
        self.dimensions.1
    }

    //---------------------------------------------------------------------------------------------
    // Returns the dimensions of the scroll log.
    //---------------------------------------------------------------------------------------------
    pub fn inner_dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    //---------------------------------------------------------------------------------------------
    // Sets the frame text at a position.
    //---------------------------------------------------------------------------------------------
    pub fn set_frame_text(&mut self, text: Option<String>, position: FrameTextPosition) {
        match position {
            FrameTextPosition::TopLeft => self.frame.top_left_text = text,
            FrameTextPosition::TopRight => self.frame.top_right_text = text,
            FrameTextPosition::BottomLeft => self.frame.bottom_left_text = text,
            FrameTextPosition::BottomRight => self.frame.bottom_right_text = text,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Clears the frame text.
    //---------------------------------------------------------------------------------------------
    pub fn clear_text(&mut self) {
        self.frame.clear_text();
    }

    //---------------------------------------------------------------------------------------------
    // Returns the total lines.
    //---------------------------------------------------------------------------------------------
    pub fn total_lines(&self) -> u32 {
        self.wrapper.total_lines()
    }

    //---------------------------------------------------------------------------------------------
    // Returns the # of lines above the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn lines_up(&self) -> u32 {
        self.wrapper.lines_up()
    }

    //---------------------------------------------------------------------------------------------
    // Returns the # of lines below the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn lines_down(&self) -> u32 {
        self.wrapper.lines_down()
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether there are any lines above the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_lines_up(&self) -> bool {
        self.wrapper.has_lines_up()
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether there are any lines below the currently visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_lines_down(&self) -> bool {
        self.wrapper.has_lines_down()
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether the content is longer than the visible area.
    //---------------------------------------------------------------------------------------------
    pub fn has_overflow(&self) -> bool {
        self.wrapper.has_overflow()
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area up by a # of lines.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_up(&mut self, lines: u32) {
        self.wrapper.scroll_up(lines);
        self.scrollbar.set_current_line(self.wrapper.lines_up());
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area down by a # of lines.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_down(&mut self, lines: u32) {
        self.wrapper.scroll_down(lines);
        self.scrollbar.set_current_line(self.wrapper.lines_up());
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area to the top.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_to_top(&mut self) {
        self.wrapper.scroll_to_top();
        self.scrollbar.set_current_line(self.wrapper.lines_up());
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Scrolls the visible area to the bottom.
    //---------------------------------------------------------------------------------------------
    pub fn scroll_to_bottom(&mut self) {
        self.wrapper.scroll_to_bottom();
        self.scrollbar.set_current_line(self.wrapper.lines_up());
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Append rich text to the scroll log.
    //---------------------------------------------------------------------------------------------
    pub fn append(&mut self, text: &str) -> Result<()> {
        self.wrapper.append(text)?;
        self.scrollbar.set_content_height(self.wrapper.total_lines());
        self.dirty = true;
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Updates the scroll log, potentially redrawing if the state changes.
    //---------------------------------------------------------------------------------------------
    pub fn update<M>(&mut self, input: &InputManager, map: &mut M) -> Result<ScrollLogAction>
    where
        M: Map2d<Tile>,
    {
        let mut action = ScrollLogAction::Noop;

        // Only update the scrollbar if the content overflows the visible area.
        let scrollbar_action = if self.has_overflow() {
            self.scrollbar.update(input, map)
        } else {
            ScrollbarAction::Noop
        };

        match scrollbar_action {
            ScrollbarAction::Focused => action = ScrollLogAction::Focused,
            ScrollbarAction::Interactable => action = ScrollLogAction::Interactable,
            ScrollbarAction::ScrollUp(lines) => {
                self.wrapper.scroll_up(lines);
                action = ScrollLogAction::Interactable;
                self.dirty = true;
            }
            ScrollbarAction::ScrollDown(lines) => {
                self.wrapper.scroll_down(lines);
                action = ScrollLogAction::Interactable;
                self.dirty = true;
            }
            _ => {}
        }

        // Redraw the wrapped text if necessary.
        if self.dirty {
            if self.has_overflow() {
                self.frame.draw(map)?;
            } else {
                self.frame.draw_clear(map)?;
            }

            self.wrapper.draw(map)?;
            self.dirty = false;
        }

        Ok(action)
    }

    //---------------------------------------------------------------------------------------------
    // Draws the scroll log. Only necessary initially and when moving the scroll log.
    //---------------------------------------------------------------------------------------------
    pub fn redraw<M>(&self, map: &mut M) -> Result<()>
    where
        M: Map2d<Tile>,
    {
        self.frame.draw(map)?;
        self.wrapper.draw(map)?;

        // Only draw the scrollbar if the content overflows the visible area.
        if self.has_overflow() {
            self.scrollbar.redraw(map);
        }

        Ok(())
    }
}
