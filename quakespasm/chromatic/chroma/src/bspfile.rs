/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2002-2009 John Fitzgibbons and others
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

use std::os::raw::{c_char, c_float, c_int, c_short, c_uint, c_ushort};
use Byte;

// upper design bounds
pub const MAX_MAP_HULLS: usize = 4;

pub const MAX_MAP_MODELS: usize = 256;
pub const MAX_MAP_BRUSHES: usize = 4096;
pub const MAX_MAP_ENTITIES: usize = 1024;
pub const MAX_MAP_ENTSTRING: usize = 65536;

pub const MAX_MAP_PLANES: usize = 32767;
pub const MAX_MAP_NODES: usize = 32767; // because negative shorts are contents
pub const MAX_MAP_CLIPNODES: usize = 32767;
pub const MAX_MAP_VERTS: usize = 65535;
pub const MAX_MAP_FACES: usize = 65535;
pub const MAX_MAP_MARKSURFACES: usize = 65535;
pub const MAX_MAP_TEXINFO: usize = 4096;
pub const MAX_MAP_EDGES: usize = 256000;
pub const MAX_MAP_SURFEDGES: usize = 512000;
pub const MAX_MAP_TEXTURES: usize = 512;
pub const MAX_MAP_MIPTEX: usize = 0x200000;
pub const MAX_MAP_LIGHTING: usize = 0x100000;
pub const MAX_MAP_VISIBILITY: usize = 0x100000;

//=============================================================================

pub const BSPVERSION: u32 = 29;

/// RMQ support (2PSB). 32bits instead of shorts for all but bbox sizes (which still use shorts)
pub const BSP2VERSION_2PSB: u32 =
    ((('B' as u32) << 24) | (('S' as u32) << 16) | (('P' as u32) << 8) | ('2' as u32));

/// BSP2 support. 32bits instead of shorts for everything (bboxes use floats)
pub const BSP2VERSION_BSP2: u32 =
    ((('B' as u32) << 0) | (('S' as u32) << 8) | (('P' as u32) << 16) | (('2' as u32) << 24));

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LumpT {
    pub fileofs: c_int,
    pub filelen: c_int,
}

impl LumpT {
    pub const fn default() -> Self {
        Self {
            fileofs: 0,
            filelen: 0,
        }
    }
}

impl Default for LumpT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum LumpType {
    Entities = 0,
    Planes = 1,
    Textures = 2,
    Vertexes = 3,
    Visibility = 4,
    Nodes = 5,
    Texinfo = 6,
    Faces = 7,
    Lighting = 8,
    ClipNodes = 9,
    Leafs = 10,
    MarkSurfaces = 11,
    Edges = 12,
    SurfEdges = 13,
    Models = 14,
}

pub const HEADER_LUMPS: usize = 15;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DModelT {
    pub mins: [c_float; 3],
    pub maxs: [c_float; 3],
    pub origin: [c_float; 3],
    pub headnode: [c_int; MAX_MAP_HULLS],
    /// not including the solid leaf 0
    pub visleafs: c_int,
    pub firstface: c_int,
    pub numfaces: c_int,
}

impl DModelT {
    pub const fn default() -> Self {
        Self {
            mins: [0.0; 3],
            maxs: [0.0; 3],
            origin: [0.0; 3],
            headnode: [0; MAX_MAP_HULLS],
            visleafs: 0,
            firstface: 0,
            numfaces: 0,
        }
    }
}

impl Default for DModelT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DHeaderT {
    pub version: c_int,
    pub lumps: [LumpT; HEADER_LUMPS],
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DMipTexLumpT {
    pub nummiptex: c_int,
    pub dataofs: [c_int; 4],
}

pub const MIPLEVELS: usize = 4;
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MipTexT {
    pub name: [c_char; 16],
    pub width: c_uint,
    pub height: c_uint,
    pub offsets: [c_uint; MIPLEVELS],
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DVertexT {
    pub point: [c_float; 3],
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum PlaneType {
    /// 0-2 are axial planes
    X = 0,
    Y = 1,
    Z = 2,
    /// 3-5 are non-axial planes snapped to the nearest
    AnyX = 3,
    AnyY = 4,
    AnyZ = 5,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct DPlaneT {
    pub normal: [c_float; 3],
    pub dist: c_float,
    // PLANE_X - PLANE_ANYZ ?remove? trivial to regenerate
    pub r#type: c_int,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum Contents {
    Empty = -1,
    Solid = -2,
    Water = -3,
    Slime = -4,
    Lava = -5,
    Sky = -6,
    // removed at csg time
    Origin = -7,
    // changed to contents_solid
    Clip = -8,

    Current0 = -9,
    Current90 = -10,
    Current180 = -11,
    Current270 = -12,
    CurrentUp = -13,
    CurrentDown = -14,
}

#[repr(C)]
pub struct DsNodeT {
    pub planenum: c_int,
    /// negative numbers are -(leafs+1), not nodes
    pub children: [c_short; 2],
    /// for sphere culling
    pub mins: [c_short; 3],
    pub maxs: [c_short; 3],
    pub firstface: c_ushort,
    /// counting both sides
    pub numfaces: c_ushort,
}

#[repr(C)]
pub struct Dl1NodeT {
    pub planenum: c_int,
    /// negative numbers are -(leafs+1), not nodes
    pub children: [c_int; 2],
    /// for sphere culling
    pub mins: [c_short; 3],
    pub maxs: [c_short; 3],
    pub firstface: c_uint,
    /// counting both sides
    pub numfaces: c_uint,
}

#[repr(C)]
pub struct Dl2NodeT {
    pub planenum: c_int,
    /// negative numbers are -(leafs+1), not nodes
    pub children: [c_int; 2],
    /// for sphere culling
    pub mins: [c_float; 3],
    pub maxs: [c_float; 3],
    pub firstface: c_uint,
    /// counting both sides
    pub numfaces: c_uint,
}

#[repr(C)]
pub struct DsClipNodeT {
    pub planenum: c_int,
    /// negative numbers are contents
    pub children: [c_short; 2],
}

#[repr(C)]
pub struct DlClipNodeT {
    pub planenum: c_int,
    /// negative numbers are contents
    pub children: [c_int; 2],
}

#[repr(C)]
pub struct TexInfoT {
    /// [s/t][xyz offset]
    pub vecs: [[c_float; 4]; 2],
    pub miptex: c_int,
    pub flags: c_int,
}

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct TexInfoFlags: c_uint {
        const None = 0;
        /// sky or slime, no lightmap or 256 subdivision
        const Special = 1 << 1;
        /// this texinfo does not have a texture
        const Missing = 1 << 2;
    }
}

// note that edge 0 is never used, because negative edge nums are used for
// counterclockwise use of the edge in a face
#[repr(C)]
pub struct DsEdgeT {
    /// vertex numbers
    pub v: [c_ushort; 2],
}

#[repr(C)]
pub struct DlEdgeT {
    /// vertex numbers
    pub v: [c_uint; 2],
}

pub const MAXLIGHTMAPS: usize = 4;

#[repr(C)]
pub struct DsFaceT {
    pub planenum: c_short,
    pub side: c_short,

    // we must support > 64k edges
    pub firstedge: c_int,
    pub numedges: c_short,
    pub texinfo: c_short,

    /// lighting info
    pub styles: [Byte; MAXLIGHTMAPS],
    /// start of [numstyles*surfsize] samples
    pub lightofs: c_int,
}

#[repr(C)]
pub struct DlFaceT {
    pub planenum: c_int,
    pub side: c_int,

    // we must support > 64k edges
    pub firstedge: c_int,
    pub numedges: c_int,
    pub texinfo: c_int,

    /// lighting info
    pub styles: [Byte; MAXLIGHTMAPS],
    /// start of [numstyles*surfsize] samples
    pub lightofs: c_int,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum AmbientSound {
    Water = 0,
    Sky = 1,
    Slime = 2,
    Lava = 3,
}

/// automatic ambient sounds
pub const NUM_AMBIENTS: usize = 4;

// leaf 0 is the generic CONTENTS_SOLID leaf, used for all solid areas
// all other leafs need visibility info
#[repr(C)]
pub struct DsLeafT {
    pub contents: c_int,
    /// -1 = no visibility info
    pub visofs: c_int,

    /// for frustum culling
    pub mins: [c_short; 3],
    pub maxs: [c_short; 3],

    pub firstmarksurface: c_short,
    pub nummarksurfaces: c_short,

    pub ambient_level: [Byte; NUM_AMBIENTS],
}

#[repr(C)]
pub struct Dl1LeafT {
    pub contents: c_int,
    /// -1 = no visibility info
    pub visofs: c_int,

    /// for frustum culling
    pub mins: [c_short; 3],
    pub maxs: [c_short; 3],

    pub firstmarksurface: c_uint,
    pub nummarksurfaces: c_uint,

    pub ambient_level: [Byte; NUM_AMBIENTS],
}

#[repr(C)]
pub struct Dl2LeafT {
    pub contents: c_int,
    /// -1 = no visibility info
    pub visofs: c_int,

    /// for frustum culling
    pub mins: [c_float; 3],
    pub maxs: [c_float; 3],

    pub firstmarksurface: c_uint,
    pub nummarksurfaces: c_uint,

    pub ambient_level: [Byte; NUM_AMBIENTS],
}
