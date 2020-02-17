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

/*
d*_t structures are on-disk representations
m*_t structures are in-memory
*/

// entity effects

use bspfile::{DModelT, MAXLIGHTMAPS, MAX_MAP_HULLS, MIPLEVELS, NUM_AMBIENTS};
use client::MAX_DLIGHTS_BITS;
use gl::types::*;
use gl_texmgr::GlTextureT;
use libc::intptr_t;
use modelgen::{SyncTypeT, TriVertexT};
use render::EFragT;
use spritegn::SpriteFrameTypeT;
use std::os::raw::{c_char, c_float, c_int, c_schar, c_short, c_uint, c_ushort};
use std::ptr::null_mut;
use zone::CacheUserT;
use VecT;
use {Byte, QBoolean, Vec3T, MAX_QPATH};

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct EntityEffects: c_uint {
        const BrightField = 1 << 1;
        const MuzzleFlash = 1 << 2;
        const BrightLight = 1 << 3;
        const DimLight = 1 << 4;
    }
}

//
// in memory representation
//
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MVertexT {
    pub position: Vec3T,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum SkySide {
    Front = 0,
    Back = 1,
    On = 2,
}

impl Default for SkySide {
    fn default() -> Self {
        SkySide::Front
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MPlaneT {
    pub normal: Vec3T,
    pub dist: c_float,
    /// for texture axis selection and fast side tests
    pub r#type: Byte,
    /// signx + signy<<1 + signz<<1
    pub signbits: Byte,
    pub pad: [Byte; 2],
}

/// Each texture has two chains, so we can clear the model chains without affecting the world.
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum TexChainT {
    World = 0,
    Model = 1,
}

impl Default for TexChainT {
    fn default() -> Self {
        TexChainT::World
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TextureT {
    pub name: [c_char; 16],
    pub width: c_uint,
    pub height: c_uint,
    pub gltexture: *mut GlTextureT,
    /// fullbright mask texture
    pub fullbright: *mut GlTextureT,
    /// for water animation
    pub warpimage: *mut GlTextureT,
    /// update warp this frame
    pub update_warp: QBoolean,
    /// for texture chains
    pub texturechains: [*mut MSurfaceT; 2],
    /// total tenths in sequence ( 0 = no)
    pub anim_total: c_int,
    pub anim_min: c_int,
    /// time for this frame min <=time< max
    pub anim_max: c_int,
    /// in the animation sequence
    pub anim_next: *mut TextureT,
    /// bmodels in frame 1 use these
    pub alternate_anims: *mut TextureT,
    /// four mip maps stored
    pub offsets: [c_uint; MIPLEVELS],
}

impl TextureT {
    pub const fn default() -> Self {
        Self {
            name: [0; 16],
            width: 0,
            height: 0,
            gltexture: null_mut(),
            fullbright: null_mut(),
            warpimage: null_mut(),
            update_warp: QBoolean::False,
            texturechains: [null_mut(); 2],
            anim_total: 0,
            anim_min: 0,
            anim_max: 0,
            anim_next: null_mut(),
            alternate_anims: null_mut(),
            offsets: [0; MIPLEVELS],
        }
    }
}

impl Default for TextureT {
    fn default() -> Self {
        Self::default()
    }
}

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct SurfaceFlags: c_uint {
        const PlaneBack = 1 << 1;
        const DrawSky = 1 << 2;
        const DrawSprite = 1 << 3;
        const DrawTurb = 1 << 4;
        const DrawTiled = 1 << 5;
        const DrawBackground = 1 << 6;
        const Underwater = 1 << 7;
        const NoTexture = 1 << 8;
        const DrawFence = 1 << 9;
        const DrawLava = 1 << 10;
        const DrawSlime = 1 << 11;
        const DrawTele = 1 << 12;
        const DrawWater = 1 << 13;
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MEdgeT {
    pub v: [c_uint; 2],
    pub cachededgeoffset: c_uint,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MTexInfoT {
    pub vecs: [[VecT; 4]; 2],
    pub mipadjust: c_float,
    pub texture: *mut TextureT,
    pub flags: c_int,
}

impl MTexInfoT {
    pub const fn default() -> Self {
        Self {
            vecs: [[0.0; 4]; 2],
            mipadjust: 0.0,
            texture: null_mut(),
            flags: 0,
        }
    }
}

impl Default for MTexInfoT {
    fn default() -> Self {
        Self::default()
    }
}

pub const VERTEXSIZE: usize = 7;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GlPolyT {
    pub next: *mut GlPolyT,
    pub chain: *mut GlPolyT,
    pub numverts: c_int,
    /// variable sized (xyz s1t1 s2t2)
    pub verts: [[c_float; VERTEXSIZE]; 4],
}

impl GlPolyT {
    pub const fn default() -> Self {
        Self {
            next: null_mut(),
            chain: null_mut(),
            numverts: 0,
            verts: [[0.0; VERTEXSIZE]; 4],
        }
    }
}

impl Default for GlPolyT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MSurfaceT {
    /// should be drawn when node is crossed
    pub visframe: c_int,
    /// for frustum culling
    pub culled: QBoolean,
    pub mins: [c_float; 3],
    pub maxs: [c_float; 3],

    pub plane: *mut MPlaneT,
    pub flags: c_int,

    /// look up in model->surfedges[], negative numbers are backwards edges
    pub firstedge: c_int,
    pub numedges: c_int,

    pub texturemins: [c_short; 2],
    pub extents: [c_short; 2],

    /// gl lightmap coordinates
    pub light_s: c_int,
    pub light_t: c_int,

    /// multiple if warped
    pub polys: *mut GlPolyT,
    pub texturechain: *mut MSurfaceT,

    pub texinfo: *mut MTexInfoT,

    /// index of this surface's first vert in the VBO
    pub vbo_firstvert: c_int,

    /// lighting info
    pub dlightframe: c_int,
    /// int is 32 bits, need an array for MAX_DLIGHTS > 32
    pub dlightbits: [c_uint; MAX_DLIGHTS_BITS],

    pub lightmaptexturenum: c_int,
    pub styles: [Byte; MAXLIGHTMAPS],
    /// values currently used in lightmap
    pub cached_light: [c_int; MAXLIGHTMAPS],
    /// true if dynamic light in cache
    pub cached_dlight: QBoolean,
    /// [numstyles*surfsize]
    pub samples: *mut Byte,
}

impl MSurfaceT {
    pub const fn default() -> Self {
        Self {
            visframe: 0,
            culled: QBoolean::False,
            mins: [0.0; 3],
            maxs: [0.0; 3],
            plane: null_mut(),
            flags: 0,
            firstedge: 0,
            numedges: 0,
            texturemins: [0; 2],
            extents: [0; 2],
            light_s: 0,
            light_t: 0,
            polys: null_mut(),
            texturechain: null_mut(),
            texinfo: null_mut(),
            vbo_firstvert: 0,
            dlightframe: 0,
            dlightbits: [0; MAX_DLIGHTS_BITS],
            lightmaptexturenum: 0,
            styles: [0; MAXLIGHTMAPS],
            cached_light: [0; MAXLIGHTMAPS],
            cached_dlight: QBoolean::False,
            samples: null_mut(),
        }
    }
}

impl Default for MSurfaceT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MNodeT {
    /// common with leaf
    /// 0, to differentiate from leafs
    pub contents: c_int,
    /// node needs to be traversed if current
    pub visframe: c_int,

    /// for bounding box culling
    pub minmaxs: [c_float; 6],

    pub parent: *mut MNodeT,

    /// node-specific
    pub plane: *mut MPlaneT,
    pub children: [*mut MNodeT; 2],
    pub firstsurface: c_uint,
    pub numsurfaces: c_uint,
}

impl MNodeT {
    pub const fn default() -> Self {
        Self {
            contents: 0,
            visframe: 0,
            minmaxs: [0.0; 6],
            parent: null_mut(),
            plane: null_mut(),
            children: [null_mut(); 2],
            firstsurface: 0,
            numsurfaces: 0,
        }
    }
}

impl Default for MNodeT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MLeafT {
    /// common with node
    /// will be a negative contents number
    pub contents: c_int,
    /// node needs to be traversed if current
    pub visframe: c_int,

    /// for bounding box culling
    pub minmaxs: [c_float; 6],

    pub parent: *mut MNodeT,

    /// leaf-specific
    pub compressed_vis: *mut Byte,
    pub efrags: *mut EFragT,

    pub firstmarksurface: *mut *mut MSurfaceT,
    pub nummarksurfaces: c_int,
    /// BSP sequence number for leaf's contents
    pub key: c_int,
    pub ambient_sound_level: [Byte; NUM_AMBIENTS],
}

impl MLeafT {
    pub const fn default() -> Self {
        Self {
            contents: 0,
            visframe: 0,
            minmaxs: [0.0; 6],
            parent: null_mut(),
            compressed_vis: null_mut(),
            efrags: null_mut(),
            firstmarksurface: null_mut(),
            nummarksurfaces: 0,
            key: 0,
            ambient_sound_level: [0; NUM_AMBIENTS],
        }
    }
}

impl Default for MLeafT {
    fn default() -> Self {
        Self::default()
    }
}

/// for clipnodes>32k
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MClipNodeT {
    pub planenum: c_int,
    /// negative numbers are contents
    pub children: [c_int; 2],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct HullT {
    pub clipnodes: *mut MClipNodeT,
    pub planes: *mut MPlaneT,
    pub firstclipnode: c_int,
    pub lastclipnode: c_int,
    pub clip_mins: Vec3T,
    pub clip_maxs: Vec3T,
}

impl HullT {
    pub const fn default() -> Self {
        Self {
            clipnodes: null_mut(),
            planes: null_mut(),
            firstclipnode: 0,
            lastclipnode: 0,
            clip_mins: Vec3T::default(),
            clip_maxs: Vec3T::default(),
        }
    }
}

impl Default for HullT {
    fn default() -> Self {
        Self::default()
    }
}

/*
==============================================================================

SPRITE MODELS

==============================================================================
*/

// FIXME: shorten these?
#[derive(Clone, Copy)]
#[repr(C)]
pub struct MSpriteFrameT {
    pub width: c_int,
    pub height: c_int,
    pub up: c_float,
    pub down: c_float,
    pub left: c_float,
    pub right: c_float,
    pub smax: c_float,
    pub tmax: c_float, // image might be padded
    pub gltexture: *mut GlTextureT,
}

impl MSpriteFrameT {
    pub const fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            up: 0.0,
            down: 0.0,
            left: 0.0,
            right: 0.0,
            smax: 0.0,
            tmax: 0.0,
            gltexture: null_mut(),
        }
    }
}

impl Default for MSpriteFrameT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MSpriteGroupT {
    pub numframes: c_int,
    pub intervals: *mut c_float,
    pub frames: [*mut MSpriteFrameT; 1], // 0; variable-sized?
}

impl MSpriteGroupT {
    pub const fn default() -> Self {
        Self {
            numframes: 0,
            intervals: null_mut(),
            frames: [null_mut(); 1],
        }
    }
}

impl Default for MSpriteGroupT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MSpriteFrameDescT {
    pub r#type: SpriteFrameTypeT,
    pub frameptr: *mut MSpriteFrameT,
}

impl MSpriteFrameDescT {
    pub const fn default() -> Self {
        Self {
            r#type: SpriteFrameTypeT::default(),
            frameptr: null_mut(),
        }
    }
}

impl Default for MSpriteFrameDescT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MSpriteT {
    pub r#type: c_int,
    pub maxwidth: c_int,
    pub maxheight: c_int,
    pub numframes: c_int,
    pub beamlength: c_float,            // remove?
    pub frames: [MSpriteFrameDescT; 1], // 0; variable-sized?
}

/*
==============================================================================

ALIAS MODELS

Alias models are position independent, so the cache manager can move them.
==============================================================================
*/

// split out to keep vertex sizes down
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct AliasMeshT {
    pub st: [c_float; 2],
    pub vertindex: c_ushort,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MeshXyzT {
    pub xyz: [Byte; 4],
    pub normal: [c_schar; 4],
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MeshStT {
    pub st: [c_float; 2],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MAliasFrameDescT {
    pub firstpose: c_int,
    pub numposes: c_int,
    pub interval: c_float,
    pub bboxmin: TriVertexT,
    pub bboxmax: TriVertexT,
    pub frame: c_int,
    pub name: [c_char; 16],
}

impl MAliasFrameDescT {
    const fn default() -> Self {
        Self {
            firstpose: 0,
            numposes: 0,
            interval: 0.0,
            bboxmin: TriVertexT::default(),
            bboxmax: TriVertexT::default(),
            frame: 0,
            name: [0; 16],
        }
    }
}

impl Default for MAliasFrameDescT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MAliasGroupFrameDescT {
    pub bboxmin: TriVertexT,
    pub bboxmax: TriVertexT,
    pub frame: c_int,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MAliasGroupT {
    pub numframes: c_int,
    pub intervals: c_int,
    pub frames: [MAliasGroupFrameDescT; 1], // 0; variable-sized?
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MTriangleT {
    pub facesfront: c_int,
    pub vertindex: [c_int; 3],
}

pub const MAX_SKINS: usize = 32;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AliasHdrT {
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

    // used to populate vbo
    /// number of verts with unique x,y,z,s,t
    pub numverts_vbo: c_int,
    /// offset into extradata: numverts_vbo aliasmesh_t
    pub meshdesc: intptr_t,
    pub numindexes: c_int,
    /// offset into extradata: numindexes unsigned shorts
    pub indexes: intptr_t,
    /// offset into extradata: numposes*vertsperframe trivertx_t
    pub vertexes: intptr_t,

    pub numposes: c_int,
    pub poseverts: c_int,
    /// numposes*poseverts trivert_t
    pub posedata: c_int,
    /// gl command list with embedded s/t
    pub commands: c_int,
    pub gltextures: [[*mut GlTextureT; 4]; MAX_SKINS],
    pub fbtextures: [[*mut GlTextureT; 4]; MAX_SKINS],

    /// only for player skins
    pub texels: [c_int; MAX_SKINS],
    /// variable sized
    pub frames: [MAliasFrameDescT; 1], // ;0 ? since variable sized?
}

impl AliasHdrT {
    pub const fn default() -> Self {
        Self {
            ident: 0,
            version: 0,
            scale: Vec3T::default(),
            scale_origin: Vec3T::default(),
            boundingradius: 0.0,
            eyeposition: Vec3T::default(),
            numskins: 0,
            skinwidth: 0,
            skinheight: 0,
            numverts: 0,
            numtris: 0,
            numframes: 0,
            synctype: SyncTypeT::default(),
            flags: 0,
            size: 0.0,
            numverts_vbo: 0,
            meshdesc: 0,
            numindexes: 0,
            indexes: 0,
            vertexes: 0,
            numposes: 0,
            poseverts: 0,
            posedata: 0,
            commands: 0,
            gltextures: [[null_mut(); 4]; MAX_SKINS],
            fbtextures: [[null_mut(); 4]; MAX_SKINS],
            texels: [0; MAX_SKINS],
            frames: [MAliasFrameDescT::default(); 1],
        }
    }
}

impl Default for AliasHdrT {
    fn default() -> Self {
        Self::default()
    }
}

pub const MAXALIASVERTS: usize = 2000;
pub const MAXALIASFRAMES: usize = 256;
pub const MAXALIASTRIS: usize = 4096;

//===================================================================

//
// Whole model
//

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum ModTypeT {
    Brush,
    Sprite,
    Alias,
}

impl ModTypeT {
    pub const fn default() -> Self {
        Self::Brush
    }
}

impl Default for ModTypeT {
    fn default() -> Self {
        Self::default()
    }
}

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct ModelFlags: c_uint {
        const None = 0;
        /// Effects
        /// leave a trail
        const Rocket = 1 << 0;
        /// leave a trail
        const Grenade = 1 << 1;
        /// leave a trail
        const Gib = 1 << 2;
        /// rotate (bonus items)
        const Rotate = 1 << 3;
        /// green split trail
        const Tracer = 1 << 4;
        /// small blood trail
        const ZomGib = 1 << 5;
        /// orange split trail + rotate
        const Tracer2 = 1 << 6;
        /// purple trail
        const Tracer3 = 1 << 7;
        /// Rendering
        /// don't lerp when animating
        const NoLerp = 1 << 8;
        /// don't cast a shadow
        const NoShadow = 1 << 9;
        /// when fullbrights are disabled; use a hack to render this model brighter
        const FullBrightHack = 1 << 10;
        /// MarkV/QSS -- make index 255 transparent on MDLs
        const Holey = 1 << 14;
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct QModelT {
    pub name: [c_char; MAX_QPATH],
    /// path id of the game directory that this model came from
    pub path_id: c_uint,
    /// bmodels and sprites don't cache normally
    pub needload: QBoolean,

    pub r#type: ModTypeT,
    pub numframes: c_int,
    pub synctype: SyncTypeT,

    pub flags: c_int,

    ///
    /// volume occupied by the model graphics
    ///
    pub mins: Vec3T,
    pub maxs: Vec3T,
    /// bounds for entities with nonzero yaw
    pub ymins: Vec3T,
    pub ymaxs: Vec3T,
    /// bounds for entities with nonzero pitch or roll
    pub rmins: Vec3T,
    pub rmaxs: Vec3T,

    ///
    /// solid volume for clipping
    ///
    pub clipbox: QBoolean,
    pub clipmins: Vec3T,
    pub clipmaxs: Vec3T,

    ///
    /// brush model
    ///
    pub firstmodelsurface: c_int,
    pub nummodelsurfaces: c_int,

    pub numsubmodels: c_int,
    pub submodels: *mut DModelT,

    pub numplanes: c_int,
    pub planes: *mut MPlaneT,

    /// number of visible leafs, not counting 0
    pub numleafs: c_int,
    pub leafs: *mut MLeafT,

    pub numvertexes: c_int,
    pub vertexes: *mut MVertexT,

    pub numedges: c_int,
    pub edges: *mut MEdgeT,

    pub numnodes: c_int,
    pub nodes: *mut MNodeT,

    pub numtexinfo: c_int,
    pub texinfo: *mut MTexInfoT,

    pub numsurfaces: c_int,
    pub surfaces: *mut MSurfaceT,

    pub numsurfedges: c_int,
    pub surfedges: *mut c_int,

    pub numclipnodes: c_int,
    pub clipnodes: *mut MClipNodeT,

    pub nummarksurfaces: c_int,
    pub marksurfaces: *mut *mut MSurfaceT,

    pub hulls: [HullT; MAX_MAP_HULLS],

    pub numtextures: c_int,
    pub textures: *mut *mut TextureT,

    pub visdata: *mut Byte,
    pub lightdata: *mut Byte,
    pub entities: *mut c_char,

    // for Mod_DecompressVis()
    pub viswarn: QBoolean,

    pub bspversion: c_int,

    ///
    /// alias model
    ///
    pub meshvbo: GLuint,
    pub meshindexesvbo: GLuint,
    /// offset in vbo of the hdr->numindexes unsigned shorts
    pub vboindexofs: c_int,
    /// offset in vbo of hdr->numposes*hdr->numverts_vbo meshxyz_t
    pub vboxyzofs: c_int,
    /// offset in vbo of hdr->numverts_vbo meshst_t
    pub vbostofs: c_int,

    ///
    /// additional model data
    ///

    /// only access through Mod_Extradata
    pub cache: CacheUserT,
}

impl QModelT {
    pub const fn default() -> Self {
        Self {
            name: [0; MAX_QPATH],
            path_id: 0,
            needload: QBoolean::False,
            r#type: ModTypeT::default(),
            numframes: 0,
            synctype: SyncTypeT::default(),
            flags: 0,
            mins: Vec3T::default(),
            maxs: Vec3T::default(),
            ymins: Vec3T::default(),
            ymaxs: Vec3T::default(),
            rmins: Vec3T::default(),
            rmaxs: Vec3T::default(),
            clipbox: QBoolean::False,
            clipmins: Vec3T::default(),
            clipmaxs: Vec3T::default(),
            firstmodelsurface: 0,
            nummodelsurfaces: 0,
            numsubmodels: 0,
            submodels: null_mut(),
            numplanes: 0,
            planes: null_mut(),
            numleafs: 0,
            leafs: null_mut(),
            numvertexes: 0,
            vertexes: null_mut(),
            numedges: 0,
            edges: null_mut(),
            numnodes: 0,
            nodes: null_mut(),
            numtexinfo: 0,
            texinfo: null_mut(),
            numsurfaces: 0,
            surfaces: null_mut(),
            numsurfedges: 0,
            surfedges: null_mut(),
            numclipnodes: 0,
            clipnodes: null_mut(),
            nummarksurfaces: 0,
            marksurfaces: null_mut(),
            hulls: [HullT::default(); MAX_MAP_HULLS],
            numtextures: 0,
            textures: null_mut(),
            visdata: null_mut(),
            lightdata: null_mut(),
            entities: null_mut(),
            viswarn: QBoolean::False,
            bspversion: 0,
            meshvbo: 0,
            meshindexesvbo: 0,
            vboindexofs: 0,
            vboxyzofs: 0,
            vbostofs: 0,
            cache: CacheUserT::default(),
        }
    }
}

impl Default for QModelT {
    fn default() -> Self {
        Self::default()
    }
}

pub mod capi {}
