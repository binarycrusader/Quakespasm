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

// common.rs -- misc functions used in client and server
use std::os::raw::c_int;
use std::ptr::null_mut;
use {Byte, QBoolean};

// if a packfile directory differs from this, it is assumed to be hacked/modified
pub const PAK0_COUNT: usize = 339; // id1/pak0.pak - v1.0x
pub const PAK0_CRC_V100: u32 = 13900; // id1/pak0.pak - v1.00
pub const PAK0_CRC_V101: u32 = 62751; // id1/pak0.pak - v1.01
pub const PAK0_CRC_V106: u32 = 32981; // id1/pak0.pak - v1.06
pub const PAK0_CRC: u32 = PAK0_CRC_V106;
pub const PAK0_COUNT_V091: usize = 308; // id1/pak0.pak - v0.91/0.92, not supported
pub const PAK0_CRC_V091: u32 = 28804; // id1/pak0.pak - v0.91/0.92, not supported

pub const CMDLINE_LENGTH: usize = 256; // mirrored in cmd.rs

#[repr(C)]
pub struct SizeBufT {
    /// if false, do a Sys_Error
    pub allowoverflow: QBoolean,
    /// set to true if the buffer size failed
    pub overflowed: QBoolean,
    pub data: *mut Byte,
    pub maxsize: c_int,
    pub cursize: c_int,
}

impl SizeBufT {
    pub const fn default() -> Self {
        Self {
            allowoverflow: QBoolean::False,
            overflowed: QBoolean::False,
            data: null_mut(),
            maxsize: 0,
            cursize: 0,
        }
    }
}

impl Default for SizeBufT {
    fn default() -> Self {
        Self::default()
    }
}

#[repr(C)]
pub struct LinkT {
    pub prev: *mut LinkT,
    pub next: *mut LinkT,
}

impl LinkT {
    pub const fn default() -> Self {
        Self {
            prev: null_mut(),
            next: null_mut(),
        }
    }
}

impl Default for LinkT {
    fn default() -> Self {
        Self::default()
    }
}

/*

All of Quake's data access is through a hierchal file system, but the contents
of the file system can be transparently merged from several sources.

The "base directory" is the path to the directory holding the quake.exe and all
game directories.  The sys_* files pass this to host_init in quakeparms_t->basedir.
This can be overridden with the "-basedir" command line parm to allow code
debugging in a different directory.  The base directory is only used during
filesystem initialization.

The "game directory" is the first tree on the search path and directory that all
generated files (savegames, screenshots, demos, config files) will be saved to.
This can be overridden with the "-game" command line parameter.  The game
directory can never be changed while quake is executing.  This is a precacution
against having a malicious server instruct clients to write files over areas they
shouldn't.

The "cache directory" is only used during development to save network bandwidth,
especially over ISDN / T1 lines.  If there is a cache directory specified, when
a file is found by the normal search path, it will be mirrored into the cache
directory, then opened there.

FIXME:
The file "parms.txt" will be read out of the game directory and appended to the
current command line arguments to allow different games to initialize startup
parms differently.  This could be used to add a "-sspeed 22050" for the high
quality sound edition.  Because they are added at the end, they will not
override an explicit setting on the original command line.

*/

#[allow(bad_style)]
pub mod capi {
    use cvar::{CVarFlags, CVarT};
    use libc::{c_char, c_float};
    use std::ffi::CStr;
    use std::os::raw::{c_int, c_ushort};
    use std::ptr::null_mut;
    use {LinkT, CMDLINE_LENGTH};
    use {QBoolean, MAX_NUM_ARGVS};

    #[no_mangle]
    pub static mut largv: [*mut c_char; MAX_NUM_ARGVS + 1] = [null_mut(); MAX_NUM_ARGVS + 1];

    #[no_mangle]
    pub static argvdummy: &'static [u8] = b" \0";

    #[no_mangle]
    pub static mut com_token: [c_char; 1024] = [0; 1024];

    #[no_mangle]
    pub static mut com_argc: c_int = 0;
    #[no_mangle]
    pub static mut com_argv: *mut *mut c_char = null_mut();

    #[no_mangle]
    pub static mut safemode: c_int = 0;

    // CvarFlags::ServerInfo is not set as sending cmdline upon CCREQ_RULE_INFO is unsafe.
    // set to correct value in COM_CheckRegistered()
    #[no_mangle]
    pub static mut registered: CVarT = CVarT {
        name: b"registered\0".as_ptr() as *const c_char,
        string: b"1\0".as_ptr() as *const c_char,
        flags: CVarFlags::Rom,
        value: 0.0,
        default_string: b"1\0".as_ptr() as *const c_char,
        callback: None,
        next: std::ptr::null_mut(),
    };

    // CvarFlags::ServerInfo is not set as sending cmdline upon CCREQ_RULE_INFO is unsafe.
    #[no_mangle]
    pub static mut cmdline: CVarT = CVarT {
        name: b"cmdline\0".as_ptr() as *const c_char,
        string: b"\0".as_ptr() as *const c_char,
        flags: CVarFlags::Rom,
        value: 0.0,
        default_string: b"\0".as_ptr() as *const c_char,
        callback: None,
        next: std::ptr::null_mut(),
    };

    /// True if using non-Id files
    #[no_mangle]
    pub static mut com_modified: QBoolean = QBoolean::False;

    #[no_mangle]
    pub static mut fitzmode: QBoolean = QBoolean::False;

    #[no_mangle]
    pub static mut com_cmdline: [c_char; CMDLINE_LENGTH] = [0; CMDLINE_LENGTH];

    #[no_mangle]
    pub static mut standard_quake: QBoolean = QBoolean::True;
    #[no_mangle]
    pub static mut rogue: QBoolean = QBoolean::False;
    #[no_mangle]
    pub static mut hipnotic: QBoolean = QBoolean::False;

    // this graphic needs to be in the pak file to use registered features
    #[no_mangle]
    pub static pop: [c_ushort; 128] = [
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x6600,
        0x0000, 0x0000, 0x0000, 0x6600, 0x0000, 0x0000, 0x0066, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0067, 0x0000, 0x0000, 0x6665, 0x0000, 0x0000, 0x0000, 0x0000, 0x0065, 0x6600, 0x0063,
        0x6561, 0x0000, 0x0000, 0x0000, 0x0000, 0x0061, 0x6563, 0x0064, 0x6561, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0061, 0x6564, 0x0064, 0x6564, 0x0000, 0x6469, 0x6969, 0x6400, 0x0064,
        0x6564, 0x0063, 0x6568, 0x6200, 0x0064, 0x6864, 0x0000, 0x6268, 0x6563, 0x0000, 0x6567,
        0x6963, 0x0064, 0x6764, 0x0063, 0x6967, 0x6500, 0x0000, 0x6266, 0x6769, 0x6a68, 0x6768,
        0x6a69, 0x6766, 0x6200, 0x0000, 0x0062, 0x6566, 0x6666, 0x6666, 0x6666, 0x6562, 0x0000,
        0x0000, 0x0000, 0x0062, 0x6364, 0x6664, 0x6362, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0062, 0x6662, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0061, 0x6661, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x6500, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x6400, 0x0000, 0x0000, 0x0000,
    ];

    // ClearLink is used for new headnodes
    #[no_mangle]
    pub unsafe fn ClearLink(l: *mut LinkT) {
        let link = &mut *l;
        link.prev = l;
        link.next = l;
    }

    #[no_mangle]
    pub unsafe fn RemoveLink(l: *mut LinkT) {
        let link = &mut *l;
        (*link.next).prev = link.prev;
        (*link.prev).next = link.next;
    }

    #[no_mangle]
    pub unsafe fn InsertLinkBefore(l: *mut LinkT, b: *mut LinkT) {
        let link = &mut *l;
        let before = &mut *b;
        link.next = before;
        link.prev = before.prev;
        (*link.prev).next = link;
        (*link.next).prev = link;
    }

    /*
    ============================================================================

                        LIBRARY REPLACEMENT FUNCTIONS

    ============================================================================
    */

    #[no_mangle]
    pub fn q_strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int {
        if s1 == s2 {
            return 0;
        }

        let p1 = unsafe { CStr::from_ptr(s1) };
        let p1bytes = p1.to_bytes_with_nul();

        let p2 = unsafe { CStr::from_ptr(s2) };
        let p2bytes = p2.to_bytes_with_nul();

        for (c1, c2) in p1bytes.iter().zip(p2bytes.iter()) {
            let lc1 = c1.to_ascii_lowercase() as c_int;
            let lc2 = c2.to_ascii_lowercase() as c_int;
            if lc1 != lc2 {
                return lc1 - lc2;
            }
        }
        return 0;
    }

    #[no_mangle]
    pub fn Q_atof(str: *const c_char) -> c_float {
        let mut cstr = unsafe { CStr::from_ptr(str) };

        let mut str_slice = cstr.to_bytes_with_nul();
        if str_slice.len() <= 1 {
            return 0.0;
        }

        let mut is_neg = false;
        if let b'-' = str_slice[0] {
            is_neg = true;
            str_slice = &str_slice[1..];
        }

        if str_slice.len() == 2 {
            if let b'0'..=b'9' = str_slice[0] {
                return if is_neg {
                    -((str_slice[0] - b'0') as c_float)
                } else {
                    (str_slice[0] - b'0') as c_float
                };
            }
        }

        if str_slice[0] == b'\'' {
            // Single character of the form 'A'
            let rval = if is_neg {
                -(str_slice[1] as c_float)
            } else {
                str_slice[1] as c_float
            };

            return rval;
        } else if str_slice[0] == b'0' && (str_slice[1] == b'x' || str_slice[1] == b'X') {
            // base 16 number prefixed with 0x or 0X
            str_slice = &str_slice[2..];

            let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(str_slice) };
            if let Ok(s) = cstr.to_str() {
                if let Ok(v) = i32::from_str_radix(s, 16) {
                    return if is_neg { -v as c_float } else { v as c_float };
                }
            }
            return if is_neg { -0.0 } else { 0.0 };
        }

        match str_slice[0] {
            b'0'..=b'9' | b'.' => {
                // Assume decimal or float
                cstr = unsafe { CStr::from_bytes_with_nul_unchecked(str_slice) };
                if let Ok(s) = cstr.to_str() {
                    if let Ok(v) = s.parse::<f32>() {
                        return if is_neg { -v } else { v };
                    }
                }
                return if is_neg { -0.0 } else { 0.0 };
            }
            _ => {
                if is_neg {
                    -0.0
                } else {
                    0.0
                }
            }
        }
    }
}
