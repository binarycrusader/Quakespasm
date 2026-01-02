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

pub mod capi {
    use super::NUM_CON_TIMES;
    use crate::client::CActiveT;
    use crate::cvar::{CVarFlags, CVarT};
    use crate::keys::KeydestT;
    use crate::{chat_team, cls, glheight, key_dest, QBoolean, MAX_OSPATH};
    use std::os::windows::io::FromRawHandle;
    use std::os::windows::raw::HANDLE;
    use std::{
        cmp::min,
        ffi::CStr,
        fs::File,
        io::Write,
        os::raw::{c_char, c_float, c_int},
        ptr::null_mut
    };

    #[unsafe(no_mangle)]
    pub static mut con_linewidth: c_int = 0;

    #[unsafe(no_mangle)]
    pub static con_cursorspeed: c_float = 4.0;

    #[unsafe(no_mangle)]
    pub static mut con_buffersize: c_int = 0;

    #[unsafe(no_mangle)]
    pub static mut con_forcedup: QBoolean = QBoolean::False; // because no entities to refresh

    #[unsafe(no_mangle)]
    pub static mut con_totallines: c_int = 0; // total lines in console scrollback
    #[unsafe(no_mangle)]
    pub static mut con_backscroll: c_int = 0; // lines up from bottom to display
    #[unsafe(no_mangle)]
    pub static mut con_current: c_int = 0; // where next message will be printed

    #[unsafe(no_mangle)]
    pub static mut con_x: c_int = 0; // offset in current line for next print
    #[unsafe(no_mangle)]
    pub static mut con_text: *mut c_char = null_mut();

    #[unsafe(no_mangle)]
    pub static mut con_lastcenterstring: [c_char; 1024] = [0; 1024];

    /// realtime time the line was generated for transparent notify lines
    #[unsafe(no_mangle)]
    pub static mut con_times: [c_float; NUM_CON_TIMES] = [0.0; NUM_CON_TIMES];

    #[unsafe(no_mangle)]
    pub static mut con_vislines: c_int = 0;

    #[unsafe(no_mangle)]
    pub static mut logfilename: [c_char; MAX_OSPATH as usize] = [0; MAX_OSPATH as usize];
    #[unsafe(no_mangle)]
    pub static mut log_fd: c_int = -1;

    // FIXME: the strings here are mutilated by Cvar_SetQuick???
    #[unsafe(no_mangle)]
    pub static mut con_notifytime: CVarT = CVarT {
        name: b"con_notifytime\0".as_ptr() as *const c_char,
        string: b"3\0".as_ptr() as *const c_char,
        flags: CVarFlags::None,
        value: 0.0,
        default_string: b"3\0".as_ptr() as *const c_char,
        callback: None,
        next: null_mut(),
    }; //seconds

    #[unsafe(no_mangle)]
    pub static mut con_logcenterprint: CVarT = CVarT {
        name: b"con_logcenterprint\0".as_ptr() as *const c_char,
        string: b"1\0".as_ptr() as *const c_char,
        flags: CVarFlags::None,
        value: 0.0,
        default_string: b"1\0".as_ptr() as *const c_char,
        callback: None,
        next: null_mut(),
    };

    #[unsafe(no_mangle)]
    pub static mut con_debuglog: QBoolean = QBoolean::False;

    #[unsafe(no_mangle)]
    pub static mut con_initialized: QBoolean = QBoolean::False;

    // Returns a bar of the desired length, but never wider than the console includes a newline,
    // unless len >= con_linewidth.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Con_Quakebar(len: c_int) -> *const c_char {
        thread_local! {
            static BAR: std::cell::RefCell<[c_char; 42]> = const { std::cell::RefCell::new([0; 42]) };
        }

        BAR.with_borrow_mut(|bar| {
            let mut nlen = min(len, (bar.len() - 2) as c_int) as usize;
            nlen = min(nlen, con_linewidth as usize);

            bar[0] = 0o35; // group separator (left tapered end of line)
            bar[1..nlen - 1].fill(0o36); // record separator (line)
            bar[nlen - 1] = 0o37; // unit separator (right tapered end of line)

            if nlen < con_linewidth as usize {
                bar[nlen] = '\n' as c_char;
                bar[nlen + 1] = 0;
            } else {
                bar[nlen] = 0;
            }

            bar.as_ptr()
        })
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Con_Clear_f() {
        if !con_text.is_null() {
            let dst = std::slice::from_raw_parts_mut(con_text, con_buffersize as usize);
            for v in dst {
                *v = ' ' as c_char
            }
        }
        con_backscroll = 0; // if console is empty, being scrolled up is confusing
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Con_ClearNotify() {
        std::ptr::write_bytes(
            std::ptr::addr_of_mut!(con_times) as *mut c_float,
            0,
            NUM_CON_TIMES,
        );
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Con_DebugLog(msg: *const c_char) {
        if log_fd == -1 {
            return;
        }

        // It would probably be faster to simply call libc::write here; but long-term, the hope is
        // to use a native rust File object instead, so this is written closer to that.
        let h = libc::get_osfhandle(log_fd) as HANDLE;
        // This handle is being borrowed from FILE, so don't let Rust close it when done.
        let mut file = std::mem::ManuallyDrop::new(File::from_raw_handle(h));

        let logmsg = CStr::from_ptr(msg);
        if let Err(_e) = file.write(logmsg.to_bytes()) {
            eprintln!("ConDebugLog failed: {}!", _e);
            return;
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Con_Linefeed() {
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

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Con_MessageMode_f() {
        if cls.state != CActiveT::Connected || cls.demoplayback == QBoolean::True {
            return;
        }

        chat_team = QBoolean::False;
        key_dest = KeydestT::KeyMessage;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Con_MessageMode2_f() {
        if cls.state != CActiveT::Connected || cls.demoplayback == QBoolean::True {
            return;
        }

        chat_team = QBoolean::True;
        key_dest = KeydestT::KeyMessage;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn LOG_Close() {
        if log_fd != -1 {
            libc::close(log_fd);
            log_fd = -1;
        }
    }
}
