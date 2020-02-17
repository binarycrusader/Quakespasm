#![allow(non_upper_case_globals)]
#[macro_use]
extern crate bitflags;
extern crate core;
extern crate gl;
extern crate libc;
extern crate num;

pub mod bspfile;

pub mod cl_main;
pub use cl_main::capi::*;

pub mod client;
pub use client::capi::*;

pub mod common;
pub use common::*;
pub use common::capi::*;

pub mod console;
pub use console::capi::*;

pub mod crc;
pub use crc::capi::*;

pub mod cvar;
pub use cvar::capi::*;

pub mod gl_model;
pub use gl_model::capi::*;

pub mod gl_screen;
pub use gl_screen::capi::*;

pub mod gl_texmgr;
pub use gl_texmgr::capi::*;

pub mod modelgen;

pub mod host;
pub use host::capi::*;

pub mod keys;
pub use keys::capi::*;

pub mod mathlib;

pub mod net;
pub mod net_defs;
pub mod net_sys;
pub mod protocol;

pub mod q_sound;

pub mod render;

pub mod spritegn;

pub mod strl;
pub use strl::capi::*;

pub mod wad;
pub use wad::capi::*;

pub mod vid;
pub use vid::capi::*;

pub mod zone;

use std::os::raw::{c_char, c_float, c_int, c_uchar, c_void};

// Common types originally found in q_stdinc.h:
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum QBoolean {
    False = 0,
    True = 1,
}

impl QBoolean {
    pub const fn default() -> Self {
        QBoolean::False
    }
}

impl Default for QBoolean {
    fn default() -> Self {
        Self::default()
    }
}

pub type VecT = c_float;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vec3T([VecT; 3]);

impl Vec3T {
    pub const fn default() -> Self {
        Vec3T([0.0, 0.0, 0.0])
    }
}

impl Default for Vec3T {
    fn default() -> Self {
        Self::default()
    }
}

pub type Fixed8T = c_int;
pub type Fixed16T = c_int;
pub type Byte = c_uchar;

// Common consts:
pub use libc::FILENAME_MAX as MAX_OSPATH;

// Common values originally found in quakedef.h:
pub const VERSION: &'static [u8] = b"1.09\0";
pub const GLQUAKE_VERSION: &'static [u8] = b"1.00\0";
pub const FITZQUAKE_VERSION: &'static [u8] = b"0.85\0";

/// Quakespasm string format is <MAJOR>.<MINOR>.<PATCH>[-suffix]
pub const QUAKESPASM_VER_STRING: &'static [u8] = b"0.93.2-chromatic\0";

pub const GAMENAME: &'static [u8] = b"id1\0";

pub const MINIMUM_MEMORY: u32 = 0x550000;
pub const MINIMUM_MEMORY_LEVELPAK: u32 = MINIMUM_MEMORY + 0x100000;

pub const MAX_NUM_ARGVS: usize = 50;

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum ViewAngles {
    /// up / down
    Pitch = 0,
    /// left / right
    Yaw = 1,
    /// fall over
    Roll = 2,
}

/// max length of a quake game pathname
pub const MAX_QPATH: usize = 64;

/// point on plane side epsilon
pub const ON_EPSILON: f32 = 0.1;

/// 1/32 epsilon to keep floating point happy (moved from world.c)
pub const DIST_EPSILON: f32 = 0.03125;

/// max length of a reliable message
pub const MAX_MSGLEN: usize = 64000;
/// max length of unreliable message
pub const MAX_DATAGRAM: usize = 32000;
/// johnfitz -- actual limit for unreliable messages to nonlocal clients
pub const DATAGRAM_MTU: u32 = 1400;

///
/// per-level limits
///
/// lowest allowed value for max_edicts cvar
pub const MIN_EDICTS: usize = 256;
/// highest allowed value for max_edicts cvar
/// ents past 8192 can't play sounds in the standard protocol
pub const MAX_EDICTS: usize = 32000;
pub const MAX_LIGHTSTYLES: usize = 64;
pub const MAX_MODELS: usize = 2048;
pub const MAX_SOUNDS: usize = 2048;

pub const SAVEGAME_COMMENT_LENGTH: usize = 39;

pub const MAX_STYLESTRING: usize = 64;

///
/// stats are integers communicated to the client by the server
///
pub const MAX_CL_STATS: usize = 32;
pub const STAT_HEALTH: u32 = 0;
pub const STAT_FRAGS: u32 = 1;
pub const STAT_WEAPON: u32 = 2;
pub const STAT_AMMO: u32 = 3;
pub const STAT_ARMOR: u32 = 4;
pub const STAT_WEAPONFRAME: u32 = 5;
pub const STAT_SHELLS: u32 = 6;
pub const STAT_NAILS: u32 = 7;
pub const STAT_ROCKETS: u32 = 8;
pub const STAT_CELLS: u32 = 9;
pub const STAT_ACTIVEWEAPON: u32 = 10;
pub const STAT_TOTALSECRETS: u32 = 11;
pub const STAT_TOTALMONSTERS: u32 = 12;
/// bumped on client side by svc_foundsecret
pub const STAT_SECRETS: u32 = 13;
/// bumped by svc_killedmonster
pub const STAT_MONSTERS: u32 = 14;

///
/// stock item defines
///
pub const IT_SHOTGUN: u32 = 1;
pub const IT_SUPER_SHOTGUN: u32 = 2;
pub const IT_NAILGUN: u32 = 4;
pub const IT_SUPER_NAILGUN: u32 = 8;
pub const IT_GRENADE_LAUNCHER: u32 = 16;
pub const IT_ROCKET_LAUNCHER: u32 = 32;
pub const IT_LIGHTNING: u32 = 64;
pub const IT_SUPER_LIGHTNING: u32 = 128;
pub const IT_SHELLS: u32 = 256;
pub const IT_NAILS: u32 = 512;
pub const IT_ROCKETS: u32 = 1024;
pub const IT_CELLS: u32 = 2048;
pub const IT_AXE: u32 = 4096;
pub const IT_ARMOR1: u32 = 8192;
pub const IT_ARMOR2: u32 = 16384;
pub const IT_ARMOR3: u32 = 32768;
pub const IT_SUPERHEALTH: u32 = 65536;
pub const IT_KEY1: u32 = 131072;
pub const IT_KEY2: u32 = 262144;
pub const IT_INVISIBILITY: u32 = 524288;
pub const IT_INVULNERABILITY: u32 = 1048576;
pub const IT_SUIT: u32 = 2097152;
pub const IT_QUAD: u32 = 4194304;
pub const IT_SIGIL1: u32 = 1 << 28;
pub const IT_SIGIL2: u32 = 1 << 29;
pub const IT_SIGIL3: u32 = 1 << 30;
pub const IT_SIGIL4: u32 = 1 << 31;

///
/// rogue changed and added defines
///
pub const RIT_SHELLS: u32 = 128;
pub const RIT_NAILS: u32 = 256;
pub const RIT_ROCKETS: u32 = 512;
pub const RIT_CELLS: u32 = 1024;
pub const RIT_AXE: u32 = 2048;
pub const RIT_LAVA_NAILGUN: u32 = 4096;
pub const RIT_LAVA_SUPER_NAILGUN: u32 = 8192;
pub const RIT_MULTI_GRENADE: u32 = 16384;
pub const RIT_MULTI_ROCKET: u32 = 32768;
pub const RIT_PLASMA_GUN: u32 = 65536;
pub const RIT_ARMOR1: u32 = 8388608;
pub const RIT_ARMOR2: u32 = 16777216;
pub const RIT_ARMOR3: u32 = 33554432;
pub const RIT_LAVA_NAILS: u32 = 67108864;
pub const RIT_PLASMA_AMMO: u32 = 134217728;
pub const RIT_MULTI_ROCKETS: u32 = 268435456;
pub const RIT_SHIELD: u32 = 536870912;
pub const RIT_ANTIGRAV: u32 = 1073741824;
pub const RIT_SUPERHEALTH: u32 = 2147483648;

///
/// hipnotic added defines
///
pub const HIT_PROXIMITY_GUN_BIT: u32 = 16;
pub const HIT_MJOLNIR_BIT: u32 = 7;
pub const HIT_LASER_CANNON_BIT: u32 = 23;
pub const HIT_PROXIMITY_GUN: u32 = (1 << HIT_PROXIMITY_GUN_BIT);
pub const HIT_MJOLNIR: u32 = (1 << HIT_MJOLNIR_BIT);
pub const HIT_LASER_CANNON: u32 = (1 << HIT_LASER_CANNON_BIT);
pub const HIT_WETSUIT: u32 = (1 << (23 + 2));
pub const HIT_EMPATHY_SHIELDS: u32 = (1 << (23 + 3));

pub const MAX_SCOREBOARD: usize = 16;
pub const MAX_SCOREBOARDNAME: usize = 32;

#[repr(C)]
pub struct QuakeParmsT {
    pub basedir: *const c_char,
    /// user's directory on UNIX platforms
    /// if user directories are enabled, basedir
    /// and userdir will point to different
    /// memory locations, otherwise to the same.
    pub userdir: *const c_char,
    pub argc: c_int,
    pub argv: *mut *mut c_char,
    pub membase: *mut c_void,
    pub memsize: c_int,
    pub numcpus: c_int,
    pub errstate: c_int,
}

#[repr(C)]
pub struct FilelistItemT {
    pub name: [c_char; 32],
    pub next: *mut FilelistItemT,
}
