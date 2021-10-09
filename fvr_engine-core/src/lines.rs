//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::cmp::max;
use std::mem::swap;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// Magic number for DDA line algorithm. This should be one of: 0x3fff, 0x7fff, or 0xbfff.
const DDA_MODIFIER: i32 = 0xbfff;

//-------------------------------------------------------------------------------------------------
// Lines provides a static API of functions for drawing lines.
//-------------------------------------------------------------------------------------------------
pub struct Lines;

impl Lines {
    //---------------------------------------------------------------------------------------------
    // Pushes the points of a line between two points into a vec using Bresenham's algorithm.
    // Adapted from https://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm.
    //---------------------------------------------------------------------------------------------
    pub fn push_bresenham(mut start: ICoord, mut end: ICoord, line: &mut Vec<ICoord>) {
        // Check the slope and potentially reorganize the points.
        let steep = (end.1 - start.1).abs() > (end.0 - start.0).abs();

        if steep {
            start = (start.1, start.0);
            end = (end.1, end.0);
        }

        if start.0 > end.0 {
            swap(&mut start, &mut end);
        }

        // Setup initial state.
        let dx = end.0 - start.0;
        let dy = (end.1 - start.1).abs();
        let y_step = if start.1 < end.1 { 1 } else { -1 };
        let mut err = dx / 2;
        let mut y = start.1;

        // Calculate the line.
        for x in start.0..=end.0 {
            // Push the new point.
            if steep {
                line.push((y, x));
            } else {
                line.push((x, y));
            }

            // Update the y coord.
            err -= dy;

            if err < 0 {
                y += y_step;
                err += dx;
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the points of a line between two points in a new vec using Bresenham's algorithm.
    // Adapted from https://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm.
    //---------------------------------------------------------------------------------------------
    pub fn bresenham(start: ICoord, end: ICoord) -> Vec<ICoord> {
        let mut line = Vec::new();
        Self::push_bresenham(start, end, &mut line);
        line
    }

    //---------------------------------------------------------------------------------------------
    // Pushes the points of a line between two points into a vec using the DDA algorithm.
    // Adapted from Squidlib.
    //---------------------------------------------------------------------------------------------
    pub fn push_dda(start: ICoord, end: ICoord, line: &mut Vec<ICoord>) {
        // Calculate some initial metrics for the line.
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let nx = dx.abs();
        let ny = dy.abs();
        let mn = max(nx, ny);

        // Push and return if line is a single point.
        if mn == 0 {
            line.push(start);
            return;
        }

        // Push points if line is purely horizontal or vertical.
        if nx == 0 {
            if dy > 0 {
                for y in start.1..=end.1 {
                    line.push((start.0, y));
                }
            } else {
                for y in (end.1..=start.1).rev() {
                    line.push((start.0, y));
                }
            }

            return;
        }

        if ny == 0 {
            if dx > 0 {
                for x in start.0..=end.0 {
                    line.push((x, start.1));
                }
            } else {
                for x in (end.0..=start.0).rev() {
                    line.push((x, start.1));
                }
            }

            return;
        }

        // Calculate from octant.
        let delta;
        let mut frac = 0;
        let y_oct = if dy < 0 { 4 } else { 0 };
        let x_oct = if dx < 0 { 2 } else { 0 };
        let n_oct = if ny > nx { 1 } else { 0 };
        let octant = y_oct | x_oct | n_oct;

        match octant {
            // +x, +y.
            0 => {
                delta = (ny << 16) / nx;

                for x in start.0..=end.0 {
                    line.push((x, start.1 + ((frac + DDA_MODIFIER) >> 16)));
                    frac += delta;
                }
            }
            1 => {
                delta = (nx << 16) / ny;

                for y in start.1..=end.1 {
                    line.push((start.0 + ((frac + DDA_MODIFIER) >> 16), y));
                    frac += delta;
                }
            }
            // -x, +y.
            2 => {
                delta = (ny << 16) / nx;

                for x in (end.0..=start.0).rev() {
                    line.push((x, start.1 + ((frac + DDA_MODIFIER) >> 16)));
                    frac += delta;
                }
            }
            3 => {
                delta = (nx << 16) / ny;

                for y in start.1..=end.1 {
                    line.push((start.0 - ((frac + DDA_MODIFIER) >> 16), y));
                    frac += delta;
                }
            }
            // -x, -y.
            6 => {
                delta = (ny << 16) / nx;

                for x in (end.0..=start.0).rev() {
                    line.push((x, start.1 - ((frac + DDA_MODIFIER) >> 16)));
                    frac += delta;
                }
            }
            7 => {
                delta = (nx << 16) / ny;

                for y in (end.1..=start.1).rev() {
                    line.push((start.0 - ((frac + DDA_MODIFIER) >> 16), y));
                    frac += delta;
                }
            }
            // +x, -y;
            4 => {
                delta = (ny << 16) / nx;

                for x in start.0..=end.0 {
                    line.push((x, start.1 - ((frac + DDA_MODIFIER) >> 16)));
                    frac += delta;
                }
            }
            5 => {
                delta = (nx << 16) / ny;

                for y in (end.1..=start.1).rev() {
                    line.push((start.0 + ((frac + DDA_MODIFIER) >> 16), y));
                    frac += delta;
                }
            }
            other => {
                panic!("Invalid octant: {}", other);
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the points of a line between two points in a new vec using the DDA algorithm.
    // Adapted from Squidlib and GoRogue libraries.
    //---------------------------------------------------------------------------------------------
    pub fn dda(start: ICoord, end: ICoord) -> Vec<ICoord> {
        let mut line = Vec::new();
        Self::push_dda(start, end, &mut line);
        line
    }
}
