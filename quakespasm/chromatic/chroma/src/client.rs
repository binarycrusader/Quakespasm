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
use libc::FILE;
use protocol::UserCmdT;
use q_sound::SfxT;
use render::{EFragT, EntityT};
use std::os::raw::{c_char, c_double, c_float, c_int, c_uint};
use std::ptr::null_mut;
use vid::VID_GRADES;
use ::{MAX_SOUNDS, SizeBufT};
use {Byte, QBoolean, Vec3T, MAX_SCOREBOARDNAME, MAX_STYLESTRING};
use {MAX_CL_STATS, MAX_MODELS};
use net_defs::QSocketT;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LightstyleT {
    pub length: c_int,
    pub map: [c_char; MAX_STYLESTRING],
    pub average: c_char,
    pub peak: c_char,
}

impl LightstyleT {
    pub const fn default() -> Self {
        Self {
            length: 0,
            map: [0; MAX_STYLESTRING],
            average: 0,
            peak: 0,
        }
    }
}

impl Default for LightstyleT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ScoreboardT {
    pub name: [c_char; MAX_SCOREBOARDNAME],
    pub entertime: c_float,
    pub frags: c_int,
    /// two 4 bit fields
    pub colors: c_int,
    pub translations: [Byte; VID_GRADES * 256],
}

impl Default for ScoreboardT {
    fn default() -> Self {
        Self {
            name: [0; MAX_SCOREBOARDNAME],
            entertime: 0.0,
            frags: 0,
            colors: 0,
            translations: [0; VID_GRADES * 256],
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CShiftT {
    pub destcolor: [c_int; 3],
    /// 0-256
    pub percent: c_int,
}

impl CShiftT {
    pub const fn default() -> Self {
        Self {
            destcolor: [0; 3],
            percent: 0,
        }
    }
}

impl Default for CShiftT {
    fn default() -> Self {
        Self::default()
    }
}

pub const CSHIFT_CONTENTS: u32 = 0;
pub const CSHIFT_DAMAGE: u32 = 1;
pub const CSHIFT_BONUS: u32 = 2;
pub const CSHIFT_POWERUP: u32 = 3;
pub const NUM_CSHIFTS: usize = 4;

///
/// client_state_t should hold all pieces of the client state
///

/// signon messages to receive before connected
pub const SIGNONS: u32 = 4;

pub const MAX_DLIGHTS: usize = 64;
pub const MAX_DLIGHTS_BITS: usize = (MAX_DLIGHTS + 31) >> 5;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DlightT {
    pub origin: Vec3T,
    pub radius: c_float,
    /// stop lighting after this time
    pub die: c_float,
    /// drop this each second
    pub decay: c_float,
    /// don't add when contributing less
    pub minlight: c_float,
    pub key: c_int,
    /// lit support via lordhavoc
    pub color: Vec3T,
}

impl DlightT {
    pub const fn default() -> Self {
        Self {
            origin: Vec3T::default(),
            radius: 0.0,
            die: 0.0,
            decay: 0.0,
            minlight: 0.0,
            key: 0,
            color: Vec3T::default(),
        }
    }
}

impl Default for DlightT {
    fn default() -> Self {
        Self::default()
    }
}

pub const MAX_BEAMS: usize = 32;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct BeamT {
    pub entity: c_int,
    pub model: *mut QModelT,
    pub endtime: c_float,
    pub start: Vec3T,
    pub end: Vec3T,
}

impl BeamT {
    pub const fn default() -> Self {
        Self {
            entity: 0,
            model: null_mut(),
            endtime: 0.0,
            start: Vec3T::default(),
            end: Vec3T::default(),
        }
    }
}

impl Default for BeamT {
    fn default() -> Self {
        Self::default()
    }
}

pub const MAX_MAPSTRING: usize = 2048;
pub const MAX_DEMOS: usize = 8;
pub const MAX_DEMONAME: usize = 16;

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum CActiveT {
    /// a dedicated server with no ability to start a client
    Dedicated,
    /// full screen console with no connection
    Disconnected,
    /// valid netcon, talking to a server
    Connected,
}

impl CActiveT {
    pub const fn default() -> Self {
        CActiveT::Dedicated
    }
}

impl Default for CActiveT {
    fn default() -> Self {
        Self::default()
    }
}

///
/// the ClientStaticT structure is persistent through an arbitrary number
/// of server connections
///
#[repr(C)]
pub struct ClientStaticT {
    pub state: CActiveT,

    /// personalization data sent to server to restart a level
    pub spawnparms: [c_char; MAX_MAPSTRING],

    /// demo loop control:
    pub demonum: c_int,
    /// -1 = don't play demos
    pub demos: [[c_char; MAX_DEMONAME]; MAX_DEMOS],
    /// when not playing

    /// demo recording info must be here, because record is started before
    /// entering a map (and clearing client_state_t)
    pub demorecording: QBoolean,
    pub demoplayback: QBoolean,

    /// did the user pause demo playback? (separate from cl.paused because we don't
    /// want a svc_setpause inside the demo to actually pause demo playback).
    pub demopaused: QBoolean,

    pub timedemo: QBoolean,
    pub forcetrack: c_int,
    /// -1 = use normal cd track
    pub demofile: *mut FILE,
    pub td_lastframe: c_int,
    /// to meter out one message a frame
    pub td_startframe: c_int,
    /// host_framecount at start
    pub td_starttime: c_float,
    /// realtime at second frame of timedemo

    /// connection information; 0 to SIGNONS
    pub signon: c_int,
    pub netcon: *mut QSocketT,
    /// writing buffer to send to server
    pub message: SizeBufT,
}

impl ClientStaticT {
    pub const fn default() -> Self {
        Self {
            state: CActiveT::Dedicated,
            spawnparms: [0; MAX_MAPSTRING],
            demonum: 0,
            demos: [[0; MAX_DEMONAME]; MAX_DEMOS],
            demorecording: QBoolean::False,
            demoplayback: QBoolean::False,
            demopaused: QBoolean::False,
            timedemo: QBoolean::False,
            forcetrack: 0,
            demofile: null_mut(),
            td_lastframe: 0,
            td_startframe: 0,
            td_starttime: 0.0,
            signon: 0,
            netcon: null_mut(),
            message: SizeBufT::default(),
        }
    }
}

impl Default for ClientStaticT {
    fn default() -> Self {
        Self::default()
    }
}

//
// the client_state_t structure is wiped completely at every
// server signon
//
#[repr(C)]
pub struct ClientStateT {
    /// since connecting to this server throw out the first couple, so the player doesn't
    /// accidentally do something the first frame
    pub movemessages: c_int,
    /// last command sent to the server
    pub cmd: UserCmdT,

    /// information for local display
    /// health, etc.
    pub stats: [c_int; MAX_CL_STATS],
    /// inventory bit flags
    pub items: c_int,
    /// cl.time of acquiring item, for blinking
    pub item_gettime: [c_float; 32],
    /// use anim frame if cl.time < this
    pub faceanimtime: c_float,

    /// color shifts for damage, powerups and content types
    pub cshifts: [CShiftT; NUM_CSHIFTS],
    pub prev_cshifts: [CShiftT; NUM_CSHIFTS],

    /// The client maintains its own idea of view angles, which are sent to the server each frame.
    /// The server sets punchangle when the view is temporarliy offset, and an angle reset commands
    /// at the start of each level and after teleporting.

    /// during demo playback viewangles is lerped between these
    pub mviewangles: [Vec3T; 2],
    pub viewangles: Vec3T,

    /// update by server, used for lean+bob (0 is newest)
    pub mvelocity: [Vec3T; 2],
    /// lerped between mvelocity[0] and [1]
    pub velocity: Vec3T,

    /// temporary offset
    pub punchangle: Vec3T,

    /// pitch drifting vars
    pub idealpitch: c_float,
    pub pitchvel: c_float,
    pub nodrift: QBoolean,
    pub driftmove: c_float,
    pub laststop: c_double,

    pub viewheight: c_float,
    /// local amount for smoothing stepups
    pub crouch: c_float,

    /// sent over by server
    pub paused: QBoolean,
    pub onground: QBoolean,
    pub inwater: QBoolean,

    /// don't change view angle, full screen, etc.
    pub intermission: c_int,
    /// latched at intermission start
    pub completed_time: c_int,

    /// the timestamp of last two messages
    pub mtime: [c_double; 2],
    /// clients view of time, should be between servertime and oldservertime to generate a lerp
    /// point for other data
    pub time: c_double,
    /// previous cl.time, time-oldtime is used to decay light values and smooth step ups
    pub oldtime: c_double,

    /// (realtime) for net trouble icon
    pub last_received_message: c_float,

    ///
    /// information that is static for the entire time connected to a server
    ///
    pub model_precache: [*mut QModelT; MAX_MODELS],
    pub sound_precache: [*mut SfxT; MAX_SOUNDS],

    pub mapname: [c_char; 128],
    /// for display on solo scoreboard
    pub levelname: [c_char; 128],
    /// cl_entitites[cl.viewentity] = player
    pub viewentity: c_int,
    pub maxclients: c_int,
    pub gametype: c_int,

    /// refresh related state
    /// cl_entitites[0].model
    pub worldmodel: *mut QModelT,
    pub free_efrags: *mut EFragT,
    pub num_efrags: c_int,
    /// held in cl_entities array
    pub num_entities: c_int,
    /// held in cl_staticentities array
    pub num_statics: c_int,
    /// the gun model
    pub viewent: EntityT,

    /// cd audio
    pub cdtrack: c_int,
    pub looptrack: c_int,

    /// frag scoreboard [cl.maxclients]
    pub scores: *mut ScoreboardT,

    pub protocol: c_uint,
    pub protocolflags: c_uint,
}

impl ClientStateT {
    pub const fn default() -> Self {
        Self {
            movemessages: 0,
            cmd: UserCmdT::default(),
            stats: [0; MAX_CL_STATS],
            items: 0,
            item_gettime: [0.0; 32],
            faceanimtime: 0.0,
            cshifts: [CShiftT::default(); NUM_CSHIFTS],
            prev_cshifts: [CShiftT::default(); NUM_CSHIFTS],
            mviewangles: [Vec3T::default(); 2],
            viewangles: Vec3T::default(),
            mvelocity: [Vec3T::default(); 2],
            velocity: Vec3T::default(),
            punchangle: Vec3T::default(),
            idealpitch: 0.0,
            pitchvel: 0.0,
            nodrift: QBoolean::False,
            driftmove: 0.0,
            laststop: 0.0,
            viewheight: 0.0,
            crouch: 0.0,
            paused: QBoolean::False,
            onground: QBoolean::False,
            inwater: QBoolean::False,
            intermission: 0,
            completed_time: 0,
            mtime: [0.0; 2],
            time: 0.0,
            oldtime: 0.0,
            last_received_message: 0.0,
            model_precache: [null_mut(); MAX_MODELS],
            sound_precache: [null_mut(); MAX_SOUNDS],
            mapname: [0; 128],
            levelname: [0; 128],
            viewentity: 0,
            maxclients: 0,
            gametype: 0,
            worldmodel: null_mut(),
            free_efrags: null_mut(),
            num_efrags: 0,
            num_entities: 0,
            num_statics: 0,
            viewent: EntityT::default(),
            cdtrack: 0,
            looptrack: 0,
            scores: null_mut(),
            protocol: 0,
            protocolflags: 0,
        }
    }
}

impl Default for ClientStateT {
    fn default() -> Self {
        Self::default()
    }
}

pub const MAX_TEMP_ENTITIES: usize = 256;
pub const MAX_STATIC_ENTITIES: usize = 4096;
pub const MAX_VISEDICTS: usize = 4096;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct KButtonT {
    /// key nums holding it down
    pub down: [c_int; 2],
    /// low bit is down state
    pub state: c_int,
}

impl KButtonT {
    pub const fn default() -> Self {
        Self {
            down: [0; 2],
            state: 0,
        }
    }
}

impl Default for KButtonT {
    fn default() -> Self {
        Self::default()
    }
}

pub mod capi {}
