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

#[allow(unused)]
const CON_TEXTSIZE: usize = (1024 * 1024);
#[allow(unused)]
const CON_MINSIZE: usize = 16384;
#[allow(unused)]
const NUM_CON_TIMES: usize = 4;

#[allow(non_snake_case)]
pub mod capi {
    use super::NUM_CON_TIMES;
    use client::CActiveT;
    use cvar::{CVarFlags, CVarT};
    use keys::KeydestT;
    use std::os::windows::io::FromRawHandle;
    use std::os::windows::io::IntoRawHandle;
    use std::os::windows::raw::HANDLE;
    use std::{
        cmp::min,
        ffi::CStr,
        fs::File,
        io::Write,
        os::raw::{c_char, c_float, c_int},
    };
    use {chat_team, cls, glheight, key_dest, QBoolean, MAX_OSPATH};

    #[no_mangle]
    pub static mut con_linewidth: c_int = 0;

    #[no_mangle]
    pub static con_cursorspeed: c_float = 4.0;

    #[no_mangle]
    pub static mut con_buffersize: c_int = 0;

    #[no_mangle]
    pub static mut con_forcedup: QBoolean = QBoolean::False; // because no entities to refresh

    #[no_mangle]
    pub static mut con_totallines: c_int = 0; // total lines in console scrollback
    #[no_mangle]
    pub static mut con_backscroll: c_int = 0; // lines up from bottom to display
    #[no_mangle]
    pub static mut con_current: c_int = 0; // where next message will be printed

    #[no_mangle]
    pub static mut con_x: c_int = 0; // offset in current line for next print
    #[no_mangle]
    pub static mut con_text: *mut c_char = std::ptr::null_mut();

    #[no_mangle]
    pub static mut con_lastcenterstring: [c_char; 1024] = [0; 1024];

    /// realtime time the line was generated for transparent notify lines
    #[no_mangle]
    pub static mut con_times: [c_float; NUM_CON_TIMES] = [0.0; NUM_CON_TIMES];

    #[no_mangle]
    pub static mut con_vislines: c_int = 0;

    #[no_mangle]
    pub static mut logfilename: [c_char; MAX_OSPATH as usize] = [0; MAX_OSPATH as usize];
    #[no_mangle]
    pub static mut log_fd: c_int = -1;

    // FIXME: the strings here are mutilated by Cvar_SetQuick???
    #[no_mangle]
    pub static mut con_notifytime: CVarT = CVarT {
        name: b"con_notifytime\0".as_ptr() as *const c_char,
        string: b"3\0".as_ptr() as *const c_char,
        flags: CVarFlags::None,
        value: 0.0,
        default_string: b"3\0".as_ptr() as *const c_char,
        callback: None,
        next: std::ptr::null_mut(),
    }; //seconds

    #[no_mangle]
    pub static mut con_logcenterprint: CVarT = CVarT {
        name: b"con_logcenterprint\0".as_ptr() as *const c_char,
        string: b"1\0".as_ptr() as *const c_char,
        flags: CVarFlags::None,
        value: 0.0,
        default_string: b"1\0".as_ptr() as *const c_char,
        callback: None,
        next: std::ptr::null_mut(),
    };

    #[no_mangle]
    pub static mut con_debuglog: QBoolean = QBoolean::False;

    #[no_mangle]
    pub static mut con_initialized: QBoolean = QBoolean::False;

    /// Returns a bar of the desired length, but never wider than the console
    /// includes a newline, unless len >= con_linewidth.
    #[no_mangle]
    pub unsafe fn Con_Quakebar(len: c_int) -> *const c_char {
        static mut BAR: [c_char; 42] = [0; 42];

        let mut nlen = min(len, (BAR.len() - 2) as c_int) as usize;
        nlen = min(nlen, con_linewidth as usize);

        BAR[0] = 0o35; // group separator (left tapered end of line)
        BAR[1..nlen - 1].fill(0o36); // record separator (line)
        BAR[nlen - 1] = 0o37; // unit separator (right tapered end of line)

        if nlen < con_linewidth as usize {
            BAR[nlen] = '\n' as c_char;
            BAR[nlen + 1] = 0;
        } else {
            BAR[nlen] = 0;
        }

        return BAR.as_ptr();
    }

    #[no_mangle]
    pub unsafe fn Con_Clear_f() {
        if !con_text.is_null() {
            let dst = std::slice::from_raw_parts_mut(con_text, con_buffersize as usize);
            for v in dst {
                *v = ' ' as c_char
            }
        }
        con_backscroll = 0; // if console is empty, being scrolled up is confusing
    }

    #[no_mangle]
    pub unsafe fn Con_ClearNotify() {
        for v in &mut con_times {
            *v = 0.0;
        }
    }

    #[no_mangle]
    pub unsafe fn Con_DebugLog(msg: *const c_char) {
        if log_fd == -1 {
            return;
        }

        // It would probably be faster to simply call libc::write here; but long-term, the hope is
        // to use a native rust File object instead, so this is written closer to that.
        let h = libc::get_osfhandle(log_fd) as HANDLE;
        let mut file = File::from_raw_handle(h);

        let logmsg = CStr::from_ptr(msg);
        if let Err(_e) = file.write(logmsg.to_bytes()) {
            eprintln!("ConDebugLog failed: {}!", _e);
            return;
        }

        file.into_raw_handle(); // don't close the fd on drop()
    }

    #[no_mangle]
    pub unsafe fn Con_Linefeed() {
        if con_backscroll != 0 {
            con_backscroll += 1
        }

        con_backscroll = min(con_backscroll, con_totallines - (glheight >> 3) - 1);

        con_x = 0;
        con_current += 1;

        if !con_text.is_null() {
            let dst = std::slice::from_raw_parts_mut(
                con_text.offset(((con_current % con_totallines) * con_linewidth) as isize),
                con_linewidth as usize,
            );
            for v in dst {
                *v = ' ' as c_char
            }
        }
    }

    #[no_mangle]
    pub unsafe fn Con_MessageMode_f() {
        if cls.state != CActiveT::Connected || cls.demoplayback == QBoolean::True {
            return;
        }

        chat_team = QBoolean::False;
        key_dest = KeydestT::KeyMessage;
    }

    #[no_mangle]
    pub unsafe fn Con_MessageMode2_f() {
        if cls.state != CActiveT::Connected || cls.demoplayback == QBoolean::True {
            return;
        }

        chat_team = QBoolean::True;
        key_dest = KeydestT::KeyMessage;
    }

    #[no_mangle]
    pub unsafe fn LOG_Close() {
        if log_fd != -1 {
            libc::close(log_fd);
            log_fd = -1;
        }
    }
}
