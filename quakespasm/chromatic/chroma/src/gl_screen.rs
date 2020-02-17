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

// gl_screen.rs -- master for refresh, status bar, console, chat, notify, etc

/*

background clear
rendering
turtle/net/ram icons
sbar
centerprint / slow centerprint
notify lines
intermission / finale overlay
loading plaque
console
menu

required background clears
required update regions


syncronous draw mode or async
One off screen buffer, with updates either copied or xblited
Need to double buffer?


async draw will require the refresh area to be cleared, because it will be
xblited, but sync draw can just ignore it.

sync
draw

CenterPrint ()
SlowPrint ()
Screen_Update ();
Con_Printf ();

net
turn off messages option

the refresh is allways rendered, unless the console is full screen


console is:
    notify lines
    half
    full

*/

#[allow(non_upper_case_globals)]
pub mod capi {
    use std::os::raw::{c_float, c_int};

    #[no_mangle]
    pub static mut glx: c_int = 0;
    #[no_mangle]
    pub static mut gly: c_int = 0;
    #[no_mangle]
    pub static mut glwidth: c_int = 0;
    #[no_mangle]
    pub static mut glheight: c_int = 0;

    #[no_mangle]
    pub static mut scr_con_current: c_float = 0.0;
    #[no_mangle]
    pub static mut scr_conlines: c_float = 0.0; // lines of console to display
}
