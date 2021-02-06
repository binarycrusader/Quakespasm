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

use std::os::raw::{c_char, c_float, c_int};
use {Byte, Vec3T};

//
// modelgen.rs: header file for model generation program
//

// *********************************************************
// * This file must be identical in the modelgen directory *
// * and in the Quake directory, because it's used to      *
// * pass data from one to the other via model files.      *
// *********************************************************

pub const ALIAS_VERSION: u32 = 6;

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum SyncTypeT {
    Sync,
    Rand,
}

impl SyncTypeT {
    pub const fn default() -> Self {
        SyncTypeT::Sync
    }
}

impl Default for SyncTypeT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum AliasFrameTypeT {
    Single,
    Group,
}

impl AliasFrameTypeT {
    pub const fn default() -> Self {
        AliasFrameTypeT::Single
    }
}

impl Default for AliasFrameTypeT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum AliasSkinTypeT {
    Single,
    Group,
}

impl AliasSkinTypeT {
    pub const fn default() -> Self {
        AliasSkinTypeT::Single
    }
}

impl Default for AliasSkinTypeT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MdlT {
    pub ident: c_int,
    pub version: c_int,
    pub scale: Vec3T,
    pub scale_origin: Vec3T,
    pub boundingradius: c_float,
    pub eyeposition: Vec3T,
    pub numskins: c_int,
    pub skinwidth: c_int,
    pub skinheight: c_int,
    pub numverts: c_int,
    pub numtris: c_int,
    pub numframes: c_int,
    pub synctype: SyncTypeT,
    pub flags: c_int,
    pub size: c_float,
}

// TODO: could be shorts
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct StVertT {
    pub onseam: c_int,
    pub s: c_int,
    pub t: c_int,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DTriangleT {
    pub facesfront: c_int,
    pub vertindex: [c_int; 3],
}

// This mirrors trivert_t in trilib.h, is present so Quake knows how to
// load this data
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TriVertexT {
    pub v: [Byte; 3],
    pub lightnormalindex: Byte,
}

impl TriVertexT {
    pub const fn default() -> Self {
        Self {
            v: [0; 3],
            lightnormalindex: 0,
        }
    }
}

impl Default for TriVertexT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DAliasFrameT {
    /// lightnormal isn't used
    pub bboxmin: TriVertexT,
    /// lightnormal isn't used
    pub bboxmax: TriVertexT,
    /// frame name from grabbing
    pub name: [c_char; 16],
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DAliasGroupT {
    pub numframes: c_int,
    /// lightnormal isn't used
    pub bboxmin: TriVertexT,
    /// lightnormal isn't used
    pub bboxmax: TriVertexT,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DAliasSkinGroupT {
    pub numskins: c_int,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DAliasIntervalT {
    pub interval: c_float,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DAliasSkinIntervalT {
    pub interval: c_float,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DAliasFrameTypeT {
    pub r#type: AliasFrameTypeT,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DAliasSkinTypeT {
    pub r#type: AliasSkinTypeT,
}

/// little-endian "IDPO"
pub const IDPOLYHEADER: u32 =
    (('O' as u32) << 24) + (('P' as u32) << 16) + (('D' as u32) << 8) + ('I' as u32);
