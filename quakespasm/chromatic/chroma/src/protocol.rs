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

// protocol.rs -- communications protocols

use mathlib::q_rint;
use num::clamp;
use std::os::raw::{c_float, c_int, c_uchar, c_uint, c_ushort};
use Vec3T;

/// standard quake protocol
pub const PROTOCOL_NETQUAKE: u32 = 15;
/// added new protocol for fitzquake 0.85
pub const PROTOCOL_FITZQUAKE: u32 = 666;
pub const PROTOCOL_RMQ: u32 = 999;

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    /// PROTOCOL_RMQ protocol flags
    pub struct RMQProtocolFlags: c_uint {
        const None = 0;
        const Shortangle = 1 << 1;
        const FloatAngle = 1 << 2;
        const F24bitCoord = 1 << 3;
        const FloatCoord = 1 << 4;
        const EdictScale = 1 << 5;
        /// cleanup insanity with alpha
        const AlphaSanity = 1 << 6;
        const Int32Coord = 1 << 7;
        /// not supported
        const MoreFlags = 1 << 31;
    }
}

// if the high bit of the servercmd is set, the low bits are fast update flags:

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct UpdateFlags: c_uint {
        const None = 0;
        const MoreBits = 1 << 0;
        const Origin1 = 1 << 1;
        const Origin2 = 1 << 2;
        const Origin3 = 1 << 3;
        const Angle2 = 1 << 4;
        /// was U_NOLERP; renamed since it's only used for MOVETYPE_STEP
        const Step = 1 << 5;
        const Frame = 1 << 6;
        /// just differentiates from other updates
        const Signal = 1 << 7;

        /// svc_update can pass all of the fast update bits; plus more
        const Angle1 = 1 << 8;
        const Angle3 = 1 << 9;
        const Model = 1 << 10;
        const ColorMap = 1 << 11;
        const Skin = 1 << 12;
        const Effects = 1 << 13;
        const LongEntity = 1 << 14;

        /// PROTOCOL_FITZQUAKE -- new bits
        const Extend1 = 1 << 15;
        /// 1 byte; uses ENTALPHA_ENCODE; not sent if equal to baseline
        const Alpha = 1 << 16;
        /// 1 byte; this is .frame & 0xFF00 (second byte)
        const Frame2 = 1 << 17;
        /// 1 byte; this is .modelindex & 0xFF00 (second byte)
        const Model2 = 1 << 18;
        /// 1 byte; 0.0-1.0 maps to 0-255; not sent if exactly 0.1; this is
        /// ent->v.nextthink - sv.time; used for lerping
        const LerpFinish = 1 << 19;
        /// 1 byte; for PROTOCOL_RMQ PRFL_EDICTSCALE; currently read but ignored
        const Scale = 1 << 20;
        const Unused21 = 1 << 21;
        const Unused22 = 1 << 22;
        /// another byte to follow; future expansion
        const Extend2 = 1 << 23;

        /// PROTOCOL_NEHAHRA transparency
        const Trans = 1 << 15;
    }
}

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct ServerUpdateFlags: c_uint {
        const None = 0;
        const ViewHeight = 1 << 0;
        const IdealPitch = 1 << 1;
        const Punch1 = 1 << 2;
        const Punch2 = 1 << 3;
        const Punch3 = 1 << 4;
        const Velocity1 = 1 << 5;
        const Velocity2 = 1 << 6;
        const Velocity3 = 1 << 7;
        const Unused8 = 1 << 8;
        const Items = 1 << 9;
        /// no data follows; the bit is it
        const OnGround = 1 << 10;
        /// no data follows; the bit is it
        const InWater = 1 << 11;
        const WeaponFrame = 1 << 12;
        const Armor = 1 << 13;
        const Weapon = 1 << 14;
        /// PROTOCOL_FITZQUAKE -- new bits
        /// another byte to follow
        const Extend1 = 1 << 15;
        /// 1 byte; this is .weaponmodel & 0xFF00 (second byte)
        const Weapon2 = 1 << 16;
        /// 1 byte; this is .armorvalue & 0xFF00 (second byte)
        const Armor2 = 1 << 17;
        /// 1 byte; this is .currentammo & 0xFF00 (second byte)
        const Ammo2 = 1 << 18;
        /// 1 byte; this is .ammo_shells & 0xFF00 (second byte)
        const Shells2 = 1 << 19;
        /// 1 byte; this is .ammo_nails & 0xFF00 (second byte)
        const Nails2 = 1 << 20;
        /// 1 byte; this is .ammo_rockets & 0xFF00 (second byte)
        const Rockets2 = 1 << 21;
        /// 1 byte; this is .ammo_cells & 0xFF00 (second byte)
        const Cells2 = 1 << 22;
        /// another byte to follow
        const Extend2 = 1 << 23;
        /// 1 byte; this is .weaponframe & 0xFF00 (second byte)
        const WeaponFrame2 = 1 << 24;
        /// 1 byte; this is alpha for weaponmodel; uses ENTALPHA_ENCODE; not sent if ENTALPHA_DEFAULT
        const WeaponAlpha = 1 << 25;
        const Unused26 = 1 << 26;
        const Unused27 = 1 << 27;
        const Unused28 = 1 << 28;
        const Unused29 = 1 << 29;
        const Unused30 = 1 << 30;
        /// another byte to follow; future expansion
        const Extend3 = 1 << 31;
    }
}

pub const DEFAULT_SOUND_PACKET_VOLUME: u32 = 255;
pub const DEFAULT_SOUND_PACKET_ATTENUATION: f32 = 1.0;

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct SoundUpdateFlags: c_uint {
        const None = 0;
        /// a byte
        const Volume = 1 << 0;
        /// a byte
        const Attenuation = 1 << 1;
        /// PROTOCOL_FITZQUAKE -- new bits
        /// a short + byte (instead of just a short)
        const LargeEntity = 1 << 3;
        /// a short soundindex (instead of a byte)
        const LargeSound = 1 << 4;
    }
}

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    /// PROTOCOL_FITZQUAKE -- flags for entity baseline messages
    pub struct BaselineFlags: c_uint {
        const None = 0;
        /// modelindex is short instead of byte
        const LargeModel = 1 << 0;
        /// frame is short instead of byte
        const LargeFrame = 1 << 1;
        /// 1 byte; uses ENTALPHA_ENCODE; not sent if ENTALPHA_DEFAULT
        const Alpha = 1 << 2;
    }
}

/// PROTOCOL_FITZQUAKE -- alpha encoding
/// entity's alpha is "default" (i.e. water obeys r_wateralpha) -- must be zero so zeroed out memory works
pub const ENTALPHA_DEFAULT: u32 = 0;
///entity is invisible (lowest possible alpha)
pub const ENTALPHA_ZERO: u32 = 1;
/// entity is fully opaque (highest possible alpha)
pub const ENTALPHA_ONE: u32 = 255;

/// Formerly: ENTALPHA_ENCODE
/// server convert to byte to send to client
pub fn ent_alpha_encode(a: f32) -> u32 {
    if a == 0.0 {
        return ENTALPHA_DEFAULT;
    }
    return q_rint(clamp(a * 254.0 + 1.0, 1.0, 255.0));
}

/// Formerly: ENTALPHA_DECODE
///client convert to float for rendering
pub fn ent_alpha_decode(a: u32) -> f32 {
    if a == ENTALPHA_DEFAULT {
        return 1.0;
    }
    return ((a as f32) - 1.0) / 254.0;
}

/// Formerly: ENTALPHA_TOSAVE
///server convert to float for savegame
pub fn ent_alpha_to_save(a: u32) -> f32 {
    if a == ENTALPHA_DEFAULT {
        return 0.0;
    } else if a == ENTALPHA_ZERO {
        return -1.0;
    }
    return ((a as f32) - 1.0) / 254.0;
}

/// defaults for clientinfo messages
pub const DEFAULT_VIEWHEIGHT: u32 = 22;

/// game types sent by serverinfo; these determine which intermission screen plays
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum GameType {
    CoOp,
    DeathMatch,
}

//==================
// note that there are some defs.qc that mirror to these numbers
// also related to svc_strings[] in cl_parse
//==================

///
/// server to client
///
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum ServerToClientMessage {
    Bad = 0,
    Nop = 1,
    Disconnect = 2,
    /// [byte] [long]
    Updatestat = 3,
    /// [long] server version
    Version = 4,
    /// [short] entity number
    Setview = 5,
    /// <see code>
    Sound = 6,
    /// [float] server time
    Time = 7,
    /// [string] null terminated string
    Print = 8,
    /// [string] stuffed into client's console buffer; the string should be \n terminated
    Stufftext = 9,
    /// [angle3] set the view angle to this absolute value
    SetAngle = 10,
    /// [long] version
    /// [string] signon string
    /// [string]..[0]model cache
    /// [string]...[0]sounds cache
    ServerInfo = 11,
    /// [byte] [string]
    LightStyle = 12,
    /// [byte] [string]
    UpdateName = 13,
    /// [byte] [short]
    UpdateFrags = 14,
    /// <shortbits + data>
    ClientData = 15,
    /// <see code>
    StopSound = 16,
    /// [byte] [byte]
    UpdateColors = 17,
    /// [vec3] <variable>
    Particle = 18,
    Damage = 19,
    SpawnStatic = 20,
    /// Obsolete
    SpawnBinary = 21,
    SpawnBaseline = 22,
    TempEntity = 23,
    /// [byte] on / off
    SetPause = 24,
    /// [byte] used for the signon sequence
    SignOnNum = 25,
    /// [string] to put in center of the screen
    CenterPrint = 26,
    KilledMonster = 27,
    FoundSecret = 28,
    /// [coord3] [byte] samp [byte] vol [byte] aten
    SpawnStaticSound = 29,
    /// [string] music
    Intermission = 30,
    /// [string] music [string] text
    Finale = 31,
    /// [byte] track [byte] looptrack
    CdTrack = 32,
    SellScreen = 33,
    CutScene = 34,

    /// PROTOCOL_FITZQUAKE -- new server messages
    /// [string] name
    Skybox = 37,
    /// BonusFlash
    Bf = 40,
    /// [byte] density [byte] red [byte] green [byte] blue [float] time
    Fog = 41,
    /// support for large modelindex, large framenum, alpha, using flags
    SpawnBaseline2 = 42,
    /// support for large modelindex, large framenum, alpha, using flags
    SpawnStatic2 = 43,
    /// [coord3] [short] samp [byte] vol [byte] aten
    SpawnStaticSound2 = 44,
}

///
/// client to server
///
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum ClientToServerMessage {
    Bad = 0,
    Nop = 1,
    Disconnect = 2,
    /// [usercmd_t]
    Move = 3,
    /// [string] message
    StringCmd = 4,
}

//
// temp entity events
//
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum TempEntityEvent {
    /// spike hitting wall
    Spike = 0,
    /// super spike hitting wall
    SuperSpike = 1,
    /// bullet hitting wall
    Gunshot = 2,
    /// rocket explosion
    Explosion = 3,
    /// tarbaby explosion
    TarExplosion = 4,
    /// lightning bolts
    Lightning1 = 5,
    /// lightning bolts
    Lightning2 = 6,
    /// spike hitting wall
    WizSpike = 7,
    /// spike hitting wall
    KnightSpike = 8,
    /// lightning bolts
    Lightning3 = 9,
    LavaSplash = 10,
    Teleport = 11,
    /// color mapped explosion
    Explosion2 = 12,
    /// grappling hook beam
    Beam = 13,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EntityStateT {
    pub origin: Vec3T,
    pub angles: Vec3T,
    pub modelindex: c_ushort,
    pub frame: c_ushort,
    pub colormap: c_uchar,
    pub skin: c_uchar,
    pub alpha: c_uchar,
    pub effects: c_int,
}

impl EntityStateT {
    pub const fn default() -> Self {
        Self {
            origin: Vec3T::default(),
            angles: Vec3T::default(),
            modelindex: 0,
            frame: 0,
            colormap: 0,
            skin: 0,
            alpha: 0,
            effects: 0,
        }
    }
}

impl Default for EntityStateT {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct UserCmdT {
    pub viewangles: Vec3T,

    /// intended velocities
    pub forwardmove: c_float,
    pub sidemove: c_float,
    pub upmove: c_float,
}

impl UserCmdT {
    pub const fn default() -> Self {
        Self {
            viewangles: Vec3T::default(),
            forwardmove: 0.0,
            sidemove: 0.0,
            upmove: 0.0,
        }
    }
}

impl Default for UserCmdT {
    fn default() -> Self {
        Self::default()
    }
}
