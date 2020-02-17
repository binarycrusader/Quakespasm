/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2002-2009 John Fitzgibbons and others
Copyright (C) 2007-2008 Kristian Duske
Copyright (C) 2010-2014 QuakeSpasm developers

This program is free software; you can redistribute it and/or
modify it under the terms of the GNU General Public License
as published by the Free Software Foundation; either version 2
of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA  02111-1307, USA.

*/

// vid.rs -- video driver defs

use std::os::raw::c_int;
use std::ptr::null_mut;
use Byte;

pub const VID_CBITS: usize = 6;
pub const VID_GRADES: usize = (1 << VID_CBITS);
pub const GAMMA_MAX: f32 = 3.0;

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum ModeStateT {
    Uninit,
    Windowed,
    FullScreen,
}

pub type PixelT = Byte;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VRectT {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub pnext: *mut VRectT,
}

impl VRectT {
    pub const fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            pnext: null_mut(),
        }
    }
}

impl Default for VRectT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VidDefT {
    /// invisible buffer
    pub buffer: *mut PixelT,
    /// 256 * VID_GRADES size
    pub colormap: *mut PixelT,
    /// index of first fullbright color
    pub fullbright: c_int,
    /// may be > width if displayed in a window
    pub width: c_int,
    pub height: c_int,
    pub numpages: c_int,
    /// if true, recalc vid-based stuff
    pub recalc_refdef: c_int,
    pub conrowbytes: c_int,
    pub conwidth: c_int,
    pub conheight: c_int,
}

impl VidDefT {
    pub const fn default() -> Self {
        Self {
            buffer: null_mut(),
            colormap: null_mut(),
            fullbright: 0,
            width: 0,
            height: 0,
            numpages: 0,
            recalc_refdef: 0,
            conrowbytes: 0,
            conwidth: 0,
            conheight: 0,
        }
    }
}

impl Default for VidDefT {
    fn default() -> Self {
        Self::default()
    }
}

pub mod capi {}
