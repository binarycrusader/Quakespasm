/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2002-2009 John Fitzgibbons and others

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

use modelgen::SyncTypeT;
use std::os::raw::{c_float, c_int};

//
// spritegn.rs: definitions file for sprite generation program
//

// **********************************************************
// * This file must be identical in the spritegen directory *
// * and in the Quake directory, because it's used to       *
// * pass data from one to the other via .spr files.        *
// **********************************************************

//-------------------------------------------------------
// This program generates .spr sprite package files.
// The format of the files is as follows:
//
// dsprite_t file header structure
// <repeat dsprite_t.numframes times>
//   <if spritegroup, repeat dspritegroup_t.numframes times>
//     dspriteframe_t frame header structure
//     sprite bitmap
//   <else (single sprite frame)>
//     dspriteframe_t frame header structure
//     sprite bitmap
// <endrepeat>
//-------------------------------------------------------

pub const SPRITE_VERSION: u32 = 1;

// TODO: shorten these?
#[repr(C)]
pub struct DSpriteT {
    pub ident: c_int,
    pub version: c_int,
    pub r#type: c_int,
    pub boundingradius: c_float,
    pub width: c_int,
    pub height: c_int,
    pub numframes: c_int,
    pub beamlength: c_float,
    pub synctype: SyncTypeT,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum SpriteViewPosition {
    /// faces view plane; up is towards the heavens
    ParallelUpright = 0,
    /// faces camera origin; up is towards the heavens
    FacingUpright = 1,
    /// faces view plane; up is towards the top of the screen
    Parallel = 2,
    /// pitch, yaw, and roll are independent of camera
    Oriented = 3,
    /// faces view plane, but obeys roll value
    ParallelOriented = 4,
}

#[repr(C)]
pub struct DSpriteFrameT {
    pub origin: [c_int; 2],
    pub width: c_int,
    pub height: c_int,
}

#[repr(C)]
pub struct DSpriteGroupT {
    pub numframes: c_int,
}

#[repr(C)]
pub struct DSpriteIntervalT {
    pub interval: c_float,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum SpriteFrameTypeT {
    Single = 0,
    Group,
}

impl SpriteFrameTypeT {
    pub const fn default() -> Self {
        SpriteFrameTypeT::Single
    }
}

impl Default for SpriteFrameTypeT {
    fn default() -> Self {
        Self::default()
    }
}

#[repr(C)]
pub struct DSpriteFrameTypeT {
    pub r#type: SpriteFrameTypeT,
}

/// little-endian "IDSP"
pub const IDSPRITEHEADER: u32 =
    (('P' as u32) << 24) + (('S' as u32) << 16) + (('D' as u32) << 8) + ('I' as u32);
