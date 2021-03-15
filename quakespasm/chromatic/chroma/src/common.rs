/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2002-2009 John Fitzgibbons and othersr
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

/// if a packfile directory differs from this, it is assumed to be hacked/modified

/// id1/pak0.pak - v1.0x
pub const PAK0_COUNT: usize = 339;
/// id1/pak0.pak - v1.00
pub const PAK0_CRC_V100: u32 = 13900;
/// id1/pak0.pak - v1.01
pub const PAK0_CRC_V101: u32 = 62751;
/// id1/pak0.pak - v1.06
pub const PAK0_CRC_V106: u32 = 32981;
pub const PAK0_CRC: u32 = PAK0_CRC_V106;
/// id1/pak0.pak - v0.91/0.92, not supported
pub const PAK0_COUNT_V091: usize = 308;
/// id1/pak0.pak - v0.91/0.92, not supported
pub const PAK0_CRC_V091: u32 = 28804;

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
    use libc::size_t;
    use net_main::capi::net_message;
    use std::ffi::CStr;
    use std::os::raw::{c_char, c_float, c_int, c_short, c_ushort, c_void};
    use std::ptr::{null, null_mut};
    use std::slice;
    use SizeBufT;
    use {cvar_null_string, q_strlcpy};
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

    #[no_mangle]
    pub static mut host_bigendian: QBoolean = QBoolean::False;

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

                    BYTE ORDER FUNCTIONS

    ============================================================================
    */
    #[no_mangle]
    pub extern "C" fn ShortSwap(l: c_short) -> c_short {
        return l.swap_bytes();
    }

    #[no_mangle]
    pub extern "C" fn ShortNoSwap(l: c_short) -> c_short {
        return l;
    }

    #[no_mangle]
    pub extern "C" fn LongSwap(l: c_int) -> c_int {
        return l.swap_bytes();
    }

    #[no_mangle]
    pub extern "C" fn LongNoSwap(l: c_int) -> c_int {
        return l;
    }

    #[no_mangle]
    pub extern "C" fn FloatSwap(f: c_float) -> c_float {
        return f.to_bits().swap_bytes() as f32;
    }

    #[no_mangle]
    pub extern "C" fn FloatNoSwap(f: c_float) -> c_float {
        return f;
    }

    /*
    ==============================================================================

            MESSAGE IO FUNCTIONS

    Handles byte ordering and avoids alignment errors
    ==============================================================================
    */

    //
    // reading functions
    //
    #[no_mangle]
    pub static mut msg_readcount: c_int = 0;
    #[no_mangle]
    pub static mut msg_badread: QBoolean = QBoolean::False;

    #[no_mangle]
    pub extern "C" fn MSG_BeginReading() {
        unsafe {
            msg_readcount = 0;
            msg_badread = QBoolean::False;
        }
    }

    /// returns -1 and sets msg_badread if no more characters are available
    #[no_mangle]
    pub extern "C" fn MSG_ReadChar() -> c_int {
        let readcount = unsafe { msg_readcount } as usize;
        let cursize = unsafe { net_message.cursize } as usize;
        if (readcount + 1) > cursize {
            unsafe { msg_badread = QBoolean::True };
            return -1;
        }

        // Preserve sign by reading as signed type of same size.
        let net_message_data =
            unsafe { slice::from_raw_parts(net_message.data as *mut c_char, cursize) };
        let c = net_message_data[readcount] as c_int;
        unsafe { msg_readcount += 1 };
        return c;
    }

    #[no_mangle]
    pub extern "C" fn MSG_ReadByte() -> c_int {
        let readcount = unsafe { msg_readcount } as usize;
        let cursize = unsafe { net_message.cursize } as usize;
        if (readcount + 1) > cursize {
            unsafe { msg_badread = QBoolean::True };
            return -1;
        }

        let net_message_data = unsafe { slice::from_raw_parts(net_message.data, cursize) };
        let c = net_message_data[readcount] as c_int;
        unsafe { msg_readcount += 1 };
        return c;
    }

    #[no_mangle]
    pub extern "C" fn MSG_ReadShort() -> c_int {
        let readcount = unsafe { msg_readcount } as usize;
        let cursize = unsafe { net_message.cursize } as usize;
        if (readcount + 2) > cursize {
            unsafe { msg_badread = QBoolean::True };
            return -1;
        }

        let net_message_data =
            unsafe { slice::from_raw_parts(net_message.data, cursize) };
        let c = ((net_message_data[readcount] as c_int)
            + ((net_message_data[readcount + 1] as c_int) << 8)) as c_short;
        unsafe { msg_readcount += 2 };
        return c as c_int;
    }

    #[no_mangle]
    pub extern "C" fn MSG_ReadLong() -> c_int {
        let readcount = unsafe { msg_readcount } as usize;
        let cursize = unsafe { net_message.cursize } as usize;
        if (readcount + 4) > cursize {
            unsafe { msg_badread = QBoolean::True };
            return -1;
        }

        let net_message_data =
            unsafe { slice::from_raw_parts(net_message.data, cursize) };
        let c = ((net_message_data[readcount] as c_int)
            + ((net_message_data[readcount + 1] as c_int) << 8)
            + ((net_message_data[readcount + 2] as c_int) << 16)
            + ((net_message_data[readcount + 3] as c_int) << 24)) as c_int;
        unsafe { msg_readcount += 4 };
        return c as c_int;
    }

    #[no_mangle]
    pub extern "C" fn MSG_ReadFloat() -> c_float {
        let readcount = unsafe { msg_readcount } as usize;
        let cursize = unsafe { net_message.cursize } as usize;
        if (readcount + 4) > cursize {
            unsafe { msg_badread = QBoolean::True };
            return 0.0; // IOU: return NAN?
        }

        let net_message_data = unsafe { slice::from_raw_parts(net_message.data, cursize) };
        let le_bytes: [u8; 4] = [
            net_message_data[readcount],
            net_message_data[readcount + 1],
            net_message_data[readcount + 2],
            net_message_data[readcount + 3],
        ];
        let c = f32::from_le_bytes(le_bytes) as c_float;
        unsafe { msg_readcount += 4 };
        return c;
    }

    #[no_mangle]
    pub extern "C" fn MSG_ReadString() -> *const c_char {
        static mut string: [c_char; 2048] = [0; 2048];
        if let Some((last_c, dest_c)) = unsafe { string.split_last_mut() } {
            for dc in dest_c.iter_mut() {
                let c = MSG_ReadByte() as c_char;
                match c {
                    -1 | 0 => {
                        *dc = b'\0' as c_char;
                        return unsafe { string.as_ptr() };
                    }
                    _ => {
                        *dc = c;
                    }
                };
            }

            *last_c = b'\0' as c_char;
        }

        return unsafe { string.as_ptr() };
    }

    /*
    ============================================================================

                        LIBRARY REPLACEMENT FUNCTIONS

    ============================================================================
    */
    #[no_mangle]
    pub extern "C" fn q_strncasecmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
        if s1 == s2 || n == 0 {
            return 0;
        }

        let p1 = unsafe { CStr::from_ptr(s1) };
        let p1bytes = p1.to_bytes_with_nul();

        let p2 = unsafe { CStr::from_ptr(s2) };
        let p2bytes = p2.to_bytes_with_nul();

        let mut rem = n;
        for (c1, c2) in p1bytes.iter().zip(p2bytes.iter()) {
            let lc1 = c1.to_ascii_lowercase() as c_int;
            let lc2 = c2.to_ascii_lowercase() as c_int;
            rem -= 1;

            if rem == 0 || lc1 != lc2 {
                return lc1 - lc2;
            }
        }
        return 0;
    }

    #[no_mangle]
    pub extern "C" fn q_strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int {
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
    pub extern "C" fn q_strcasestr(
        haystack: *const c_char,
        needle: *const c_char,
    ) -> *const c_char {
        if haystack == needle || unsafe { *needle } == 0 {
            return haystack;
        }

        let p1 = unsafe { CStr::from_ptr(haystack) };
        let hay_bytes = p1.to_bytes();
        let hay_len = hay_bytes.len();

        let p2 = unsafe { CStr::from_ptr(needle) };
        let needle_bytes = p2.to_bytes();
        let needle_len = needle_bytes.len();

        if hay_len < needle_len {
            return null(); // Not enough to find match of expected size.
        }

        for pos in 0..hay_len {
            let rem_bytes = &hay_bytes[pos..pos + needle_len];
            if rem_bytes.eq_ignore_ascii_case(needle_bytes) {
                return unsafe { haystack.offset(pos as isize) };
            } else if (pos + 1 + needle_len) > hay_len {
                return null(); // Not enough left to find match of expected size on next iteration.
            }
        }

        return null(); // didn't find it
    }

    #[no_mangle]
    pub extern "C" fn Q_atof(str: *const c_char) -> c_float {
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

            cstr = unsafe { CStr::from_bytes_with_nul_unchecked(str_slice) };
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

    #[no_mangle]
    pub extern "C" fn Q_memset(dest: *mut c_void, fill: c_int, count: size_t) {
        // TODO: Replace with dest_slice.fill() in rust 1.50+
        let dest_slice = unsafe { slice::from_raw_parts_mut(dest as *mut u8, count) };
        for i in &mut dest_slice[..] {
            *i = fill as u8
        }
    }

    #[no_mangle]
    pub extern "C" fn Q_memcpy(dest: *mut c_void, src: *const c_void, count: size_t) {
        let src_slice = unsafe { slice::from_raw_parts_mut(src as *mut u8, count) };
        let dest_slice = unsafe { slice::from_raw_parts_mut(dest as *mut u8, count) };
        dest_slice.copy_from_slice(src_slice)
    }

    #[no_mangle]
    pub extern "C" fn Q_strcpy(dest: *mut c_char, src: *const c_char) {
        let src_str = unsafe { CStr::from_ptr(src) };
        let src_str_slice = src_str.to_bytes_with_nul();

        let dst_str_slice =
            unsafe { slice::from_raw_parts_mut(dest as *mut u8, src_str_slice.len()) };
        dst_str_slice.copy_from_slice(src_str_slice);
    }

    #[no_mangle]
    pub extern "C" fn Q_strncpy(dest: *mut c_char, src: *const c_char, count: c_int) {
        let src_str = unsafe { CStr::from_ptr(src) };
        let mut src_str_slice = src_str.to_bytes_with_nul();

        let dst_str_slice;
        if src_str_slice.len() > count as usize {
            src_str_slice = unsafe { slice::from_raw_parts_mut(src as *mut u8, count as usize) };
            dst_str_slice = unsafe { slice::from_raw_parts_mut(dest as *mut u8, count as usize) };
        } else {
            dst_str_slice =
                unsafe { slice::from_raw_parts_mut(dest as *mut u8, src_str_slice.len()) };
        }
        dst_str_slice.copy_from_slice(src_str_slice);
    }

    #[no_mangle]
    pub extern "C" fn Q_strlen(str: *const c_char) -> c_int {
        let s = unsafe { CStr::from_ptr(str) };
        s.to_bytes().len() as c_int
    }

    #[no_mangle]
    pub extern "C" fn Q_strrchr(s: *const c_char, c: c_char) -> *const c_char {
        let str = unsafe { CStr::from_ptr(s) };
        let str_slice = str.to_bytes();
        str_slice
            .iter()
            .rposition(|&hay| hay == c as u8)
            .map_or(null(), |pos| unsafe { s.add(pos) })
    }

    #[no_mangle]
    pub extern "C" fn Q_strcat(dest: *mut c_char, src: *const c_char) {
        let src_str = unsafe { CStr::from_ptr(src) };
        let src_str_slice = src_str.to_bytes_with_nul();

        let dest_str = unsafe { CStr::from_ptr(dest) };
        let dest_str_slice = unsafe {
            slice::from_raw_parts_mut(
                dest.add(dest_str.to_bytes().len()) as *mut u8,
                src_str_slice.len(),
            )
        };
        dest_str_slice.copy_from_slice(src_str_slice);
    }

    #[no_mangle]
    pub extern "C" fn Q_strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
        let mut p1 = s1;
        let mut p2 = s2;

        loop {
            unsafe {
                if *p1 != *p2 {
                    return -1;
                } // strings not equal
                if *p1 == b'\0' as i8 {
                    return 0;
                } // strings are equal
                p1 = p1.add(1);
                p2 = p2.add(1);
            }
        }
    }

    #[no_mangle]
    pub unsafe fn Q_strncmp(s1: *const c_char, s2: *const c_char, count: c_int) -> c_int {
        let mut p1 = s1;
        let mut p2 = s2;
        let mut rem = count;

        loop {
            if rem == 0 {
                return 0;
            } // strings are equal inclusive of count
            if *p1 != *p2 {
                return -1;
            } // strings not equal
            if *p1 == b'\0' as i8 {
                return 0;
            } // strings are equal
            p1 = p1.add(1);
            p2 = p2.add(1);
            rem -= 1;
        }
    }

    #[no_mangle]
    pub extern "C" fn Q_atoi(str: *const c_char) -> c_int {
        let mut cstr = unsafe { CStr::from_ptr(str) };

        let mut str_slice = cstr.to_bytes_with_nul();
        if str_slice.len() <= 1 {
            return 0;
        }

        let mut is_neg = false;
        if let b'-' = str_slice[0] {
            is_neg = true;
            str_slice = &str_slice[1..];
        }

        if str_slice.len() == 2 {
            if let b'0'..=b'9' = str_slice[0] {
                return if is_neg {
                    -((str_slice[0] - b'0') as c_int)
                } else {
                    (str_slice[0] - b'0') as c_int
                };
            }
        }

        //
        // check for character
        //
        if str_slice[0] == b'\'' {
            // Single character of the form 'A'
            let rval = if is_neg {
                -(str_slice[1] as c_int)
            } else {
                str_slice[1] as c_int
            };

            return rval;
        } else if str_slice[0] == b'0' && (str_slice[1] == b'x' || str_slice[1] == b'X') {
            // base 16 number prefixed with 0x or 0X
            str_slice = &str_slice[2..];

            let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(str_slice) };
            if let Ok(s) = cstr.to_str() {
                if let Ok(v) = i32::from_str_radix(s, 16) {
                    return if is_neg { -v as c_int } else { v as c_int };
                }
            }
            return if is_neg { -0 } else { 0 };
        }

        match str_slice[0] {
            b'0'..=b'9' => {
                // Assume decimal
                cstr = unsafe { CStr::from_bytes_with_nul_unchecked(str_slice) };
                if let Ok(s) = cstr.to_str() {
                    if let Ok(v) = s.parse::<i32>() {
                        return if is_neg { -v } else { v };
                    }
                }
                return if is_neg { -0 } else { 0 };
            }
            _ => {
                if is_neg {
                    -0
                } else {
                    0
                }
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn SZ_Clear(buf: *mut SizeBufT) {
        unsafe { (*buf).cursize = 0 };
    }

    /// Returns the position (1 to argc - 1) in the program's argument list where the given parameter
    /// appears, or 0 if not present.
    #[no_mangle]
    pub extern "C" fn COM_CheckParm(parm: *const c_char) -> c_int {
        let argc = unsafe { com_argc } as usize;
        for i in 1..argc {
            let arg = unsafe { *com_argv.add(i) };
            if arg.is_null() {
                continue; // NEXTSTEP sometimes clears appkit vars.
            }

            if Q_strcmp(parm, arg) == 0 {
                return i as c_int;
            }
        }

        return 0;
    }

    #[no_mangle]
    pub extern "C" fn COM_SkipPath(pathname: *const c_char) -> *const c_char {
        let str = unsafe { CStr::from_ptr(pathname) };
        let str_slice = str.to_bytes();
        str_slice
            .iter()
            .rposition(|&hay| hay == '/' as u8)
            .map_or(pathname, |pos| unsafe { pathname.add(pos + 1) })
    }

    #[no_mangle]
    pub extern "C" fn COM_StripExtension(inn: *const c_char, outn: *mut c_char, outsize: size_t) {
        if inn.is_null() {
            unsafe {
                outn.write_unaligned('\0' as i8);
            }
            return;
        }

        if inn != outn {
            // If not an in-place edit, duplicate source first.
            q_strlcpy(outn, inn, outsize);
        }

        let str = unsafe { CStr::from_ptr(outn) };
        let str_slice = str.to_bytes();

        for (pos, c) in str_slice.iter().enumerate().rev() {
            match *c {
                b'/' | b'\\' => return, // no extension
                b'.' => {
                    unsafe {
                        outn.add(pos).write_unaligned('\0' as i8);
                    }
                    return;
                }
                _ => continue,
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn COM_FileGetExtension(inn: *const c_char) -> *const c_char {
        let str = unsafe { CStr::from_ptr(inn) };
        let str_slice = str.to_bytes();
        for (pos, c) in str_slice.iter().enumerate().rev() {
            match c {
                b'.' => {
                    return unsafe { inn.add(pos + 1) };
                }
                b'/' | b'\\' => {
                    break;
                }
                _ => {
                    continue;
                }
            }
        }

        return cvar_null_string.as_ptr() as *const c_char; // eol; no extension
    }

    /// Given '[somedir/otherdir/]filename[.ext]', write only 'filename' to the output up to outsize
    /// - 1 characters.  If no 'filename' is present, '?model?' will be used as the 'filename' for
    /// debugging purposes.
    #[no_mangle]
    pub extern "C" fn COM_FileBase(inn: *const c_char, outn: *mut c_char, outsize: size_t) {
        // TODO: simplify
        let mut has_path = false;
        let mut has_ext = false;
        let mut pos_path = 0;
        let mut pos_ext = 0;
        if !inn.is_null() {
            let src = unsafe { CStr::from_ptr(inn) };
            let mut src_slice = src.to_bytes();

            for (pos, c) in src_slice.iter().enumerate().rev() {
                match *c {
                    b'.' => {
                        pos_ext = pos;
                        has_ext = true;
                        continue;
                    }
                    b'/' | b'\\' => {
                        pos_path = pos;
                        has_path = true;
                        break;
                    }
                    _ => continue,
                }
            }

            if has_ext {
                src_slice = src_slice.split_at(pos_ext).0;
            }

            if has_path {
                if (pos_path + 1) == src_slice.len() {
                    src_slice = &[];
                } else {
                    src_slice = src_slice.split_at(pos_path + 1).1;
                }
            }

            if !src_slice.is_empty() {
                let dest_slice;
                if src_slice.len() >= outsize {
                    src_slice = src_slice.split_at(outsize - 1).0;
                    dest_slice = unsafe { slice::from_raw_parts_mut(outn as *mut u8, outsize - 1) };
                } else {
                    dest_slice =
                        unsafe { slice::from_raw_parts_mut(outn as *mut u8, src_slice.len()) };
                }

                dest_slice.copy_from_slice(src_slice);
                unsafe {
                    outn.add(src_slice.len()).write_unaligned(b'\0' as i8);
                }
                return;
            }
        }

        // If no basename, it could be a model, so use a debug-friendly basename.
        q_strlcpy(outn, b"?model?".as_ptr() as *const c_char, outsize);
    }

    /// If 'path' is not empty, and does not have an extension or the extension doesn't match .EXT,
    /// and path plus new 'extension' does not exceed 'len' - 1, append it ('extension' should
    /// include the leading ".").
    #[no_mangle]
    pub extern "C" fn COM_AddExtension(path: *mut c_char, extension: *const c_char, len: size_t) {
        let path_str = unsafe { CStr::from_ptr(path) };
        let path_slice = path_str.to_bytes();

        if path_slice.len() == 0 || len <= path_slice.len() {
            return; // eop; cannot add extension
        }

        let ext_str = unsafe { CStr::from_ptr(extension) };
        let ext_slice = ext_str.to_bytes();

        let mut new_ext_pos = path_slice.len();
        for (pos, c) in path_slice.iter().enumerate().rev() {
            match c {
                b'.' => {
                    let path_ext_slice = &path_slice[pos..];
                    if ext_slice.eq_ignore_ascii_case(path_ext_slice) {
                        new_ext_pos = 0; // nothing to do
                    } else {
                        new_ext_pos = pos;
                    }
                    break;
                }
                b'/' | b'\\' => {
                    if (pos + 1) >= path_slice.len() {
                        return; // eop; cannot add extension
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if new_ext_pos == 0 {
            return; // abort; either path is empty or already has extension
        } else if new_ext_pos + 1 + ext_slice.len() >= len {
            return; // abort; not enough space to add new extension
        }

        let dst_slice = unsafe { slice::from_raw_parts_mut(path, len) };
        let (_, dst_rem_slice) = dst_slice.split_at_mut(new_ext_pos);
        q_strlcpy(dst_rem_slice.as_mut_ptr(), extension, dst_rem_slice.len());
        return;
    }
}
