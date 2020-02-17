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
// cl_main.rs  -- client main loop

#[allow(bad_style)]
pub mod capi {
    use crate::MAX_LIGHTSTYLES;
    use client::{
        ClientStateT, ClientStaticT, DlightT, LightstyleT, MAX_DLIGHTS, MAX_STATIC_ENTITIES,
        MAX_VISEDICTS,
    };
    use render::EntityT;
    use std::os::raw::c_int;
    use std::ptr::null_mut;

    #[no_mangle]
    pub static mut cls: ClientStaticT = ClientStaticT::default();

    #[no_mangle]
    pub static mut cl: ClientStateT = ClientStateT::default();

    // FIXME: put these on hunk?
    #[no_mangle]
    pub static mut cl_static_entities: [EntityT; MAX_STATIC_ENTITIES] =
        [EntityT::default(); MAX_STATIC_ENTITIES];

    #[no_mangle]
    pub static mut cl_lightstyle: [LightstyleT; MAX_LIGHTSTYLES] =
        [LightstyleT::default(); MAX_LIGHTSTYLES];

    #[no_mangle]
    pub static mut cl_dlights: [DlightT; MAX_DLIGHTS] = [DlightT::default(); MAX_DLIGHTS];

    /// was static array; now on hunk
    #[no_mangle]
    pub static mut cl_entities: *mut EntityT = null_mut();
    /// only changes when new map loads
    #[no_mangle]
    pub static mut cl_max_edicts: c_int = 0;

    #[no_mangle]
    pub static mut cl_numvisedicts: c_int = 0;
    #[no_mangle]
    pub static mut cl_visedicts: [*mut EntityT; MAX_VISEDICTS] = [null_mut(); MAX_VISEDICTS];
}
