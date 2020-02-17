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

use gl_model::QModelT;
use protocol::EntityStateT;
use std::os::raw::{c_double, c_float, c_int, c_short, c_uint};
use std::ptr::null_mut;
use vid::VRectT;
use QBoolean;
use {Byte, Vec3T};

/// soldier uniform colors
pub const TOP_RANGE: usize = 16;
pub const BOTTOM_RANGE: usize = 96;

//=============================================================================

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EFragT {
    pub leafnext: *mut EFragT,
    pub entity: *mut EntityT,
}

impl Default for EFragT {
    fn default() -> Self {
        EFragT {
            leafnext: null_mut(),
            entity: null_mut(),
        }
    }
}

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct LerpFlags: c_uint {
        const None = 0;
        /// this is a MOVETYPE_STEP entity, enable movement lerp
        const MoveStep = 1 << 0;
        /// disable anim lerping until next anim frame
        const ResetAnim = 1 << 1;
        ///set this and previous flag to disable anim lerping for two anim frames
        const ResetAnim2 = 1 << 2;
        /// disable movement lerping until next origin/angles change
        const ResetMove = 1 << 3;
        /// use lerpfinish time from server update instead of assuming interval of 0.1
        const Finish = 1 << 4;
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EntityT {
    /// model changed
    pub forcelink: QBoolean,

    /// to fill in defaults in updates
    pub baseline: EntityStateT,

    /// time of last update
    pub msgtime: c_double,
    /// last two updates (0 is newest)
    pub msg_origins: [Vec3T; 2],
    pub origin: Vec3T,
    /// last two updates (0 is newest)
    pub msg_angles: [Vec3T; 2],
    pub angles: Vec3T,
    /// NULL = no model
    pub model: *mut QModelT,
    /// linked list of efrags
    pub efrag: *mut EFragT,
    pub frame: c_int,
    /// for client-side animations
    pub syncbase: c_float,
    pub colormap: *mut Byte,
    /// light, particles, etc.
    pub effects: c_int,
    /// for Alias models
    pub skinnum: c_int,
    /// last frame this entity was found in an active leaf
    pub visframe: c_int,

    pub alpha: Byte,
    pub lerpflags: Byte,
    /// animation lerping
    pub lerpstart: c_float,
    pub lerptime: c_float,
    /// server sent us a more accurate interval, use it instead of 0.1
    pub lerpfinish: c_float,
    pub previouspose: c_short,
    pub currentpose: c_short,
    /// transform lerping
    pub movelerpstart: c_float,
    pub previousorigin: Vec3T,
    pub currentorigin: Vec3T,
    pub previousangles: Vec3T,
    pub currentangles: Vec3T,
}

impl EntityT {
    pub const fn default() -> Self {
        Self {
            forcelink: QBoolean::False,
            baseline: EntityStateT::default(),
            msgtime: 0.0,
            msg_origins: [Vec3T::default(); 2],
            origin: Vec3T::default(),
            msg_angles: [Vec3T::default(); 2],
            angles: Vec3T::default(),
            model: null_mut(),
            efrag: null_mut(),
            frame: 0,
            syncbase: 0.0,
            colormap: null_mut(),
            effects: 0,
            skinnum: 0,
            visframe: 0,
            alpha: 0,
            lerpflags: 0,
            lerpstart: 0.0,
            lerptime: 0.0,
            lerpfinish: 0.0,
            previouspose: 0,
            currentpose: 0,
            movelerpstart: 0.0,
            previousorigin: Vec3T::default(),
            currentorigin: Vec3T::default(),
            previousangles: Vec3T::default(),
            currentangles: Vec3T::default(),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RefDefT {
    /// subwindow in video for refresh
    pub vrect: VRectT,
    pub vieworg: Vec3T,
    pub viewangles: Vec3T,
    pub fov_x: c_float,
    pub fov_y: c_float,
}

impl RefDefT {
    pub const fn default() -> Self {
        Self {
            vrect: VRectT::default(),
            vieworg: Vec3T::default(),
            viewangles: Vec3T::default(),
            fov_x: 0.0,
            fov_y: 0.0,
        }
    }
}

impl Default for RefDefT {
    fn default() -> Self {
        Self::default()
    }
}
