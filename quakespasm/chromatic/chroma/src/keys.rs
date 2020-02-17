/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2002-2009 John Fitzgibbons and others
Copyright (C) 2007-2008 Kristian Duske
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

// key up events are sent even if in console mode

pub const MAX_KEYS: usize = 256;

pub const MAXCMDLINE: usize = 256;

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum KeydestT {
    KeyGame,
    KeyConsole,
    KeyMessage,
    KeyMenu,
}

pub const CMDLINES: usize = 64;

pub mod capi {
    use super::{CMDLINES, MAXCMDLINE, MAX_KEYS};
    use keys::KeydestT;
    use std::os::raw::{c_char, c_double, c_int};
    use std::ptr::null_mut;
    use QBoolean;

    #[no_mangle]
    pub static mut key_lines: [[c_char; CMDLINES]; MAXCMDLINE] = [[0; CMDLINES]; MAXCMDLINE];

    #[no_mangle]
    pub static mut key_linepos: c_int = 0;
    /// -- insert key toggle (for editing)
    #[no_mangle]
    pub static mut key_insert: c_int = 0;
    /// fudge cursor blinking to make it easier to spot in certain cases
    #[no_mangle]
    pub static mut key_blinktime: c_double = 0.0;

    #[no_mangle]
    pub static mut edit_line: c_int = 0;
    #[no_mangle]
    pub static mut history_line: c_int = 0;

    #[no_mangle]
    pub static mut key_dest: KeydestT = KeydestT::KeyGame;

    #[no_mangle]
    pub static mut keybindings: [*mut c_char; MAX_KEYS] = [null_mut(); MAX_KEYS];

    /// if true, can't be rebound while in console
    #[no_mangle]
    pub static mut consolekeys: [QBoolean; MAX_KEYS] = [QBoolean::False; MAX_KEYS];
    /// if true, can't be rebound while in menu
    #[no_mangle]
    pub static mut menubound: [QBoolean; MAX_KEYS] = [QBoolean::False; MAX_KEYS];
    #[no_mangle]
    pub static mut keydown: [QBoolean; MAX_KEYS] = [QBoolean::False; MAX_KEYS];

    #[no_mangle]
    pub static mut chat_team: QBoolean = QBoolean::False;
    #[no_mangle]
    pub static mut chat_buffer: [c_char; MAXCMDLINE] = [0; MAXCMDLINE];
    #[no_mangle]
    pub static mut chat_bufferlen: c_int = 0;
}
