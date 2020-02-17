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

use ::{QBoolean, Byte};
use std::os::raw::c_int;
use std::ptr::null_mut;

#[repr(C)]
pub struct SizeBufT
{
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
            cursize: 0
        }
    }
}

impl Default for SizeBufT
{
    fn default() -> Self {
        Self::default()
    }
}

#[allow(bad_style)]
pub mod capi {
    use libc::{c_char, c_float};
    use std::ffi::CStr;

    #[no_mangle]
    pub unsafe fn Q_atof(str: *const c_char) -> c_float {
        let mut cstr = CStr::from_ptr(str);

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

            let cstr = CStr::from_bytes_with_nul_unchecked(str_slice);
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
                cstr = CStr::from_bytes_with_nul_unchecked(str_slice);
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
