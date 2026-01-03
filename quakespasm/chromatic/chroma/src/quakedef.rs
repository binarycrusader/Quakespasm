/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2002-2009 John Fitzgibbons and others
Copyright (C) 2007-2008 Kristian Duske
Copyright (C) 2010-2019 QuakeSpasm developers

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

// quakedef.rs -- primary definitions for client

use std::os::raw::{c_char, c_int, c_void};

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct QuakeParmsT {
    pub basedir: *const c_char,
    // user's directory on UNIX platforms; if user directories are enabled, basedir and userdir will
    // point to different memory locations, otherwise to the same.
    pub userdir: *const c_char,
    pub argc: c_int,
    pub argv: *const *const c_char,
    pub membase: *mut c_void,
    pub memsize: *mut c_int,
    pub numcpus: *mut c_int,
    pub errstate: *mut c_int,
}
