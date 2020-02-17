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

/*
 memory allocation


H_??? The hunk manages the entire memory block given to quake.  It must be
contiguous.  Memory can be allocated from either the low or high end in a
stack fashion.  The only way memory is released is by resetting one of the
pointers.

Hunk allocations should be given a name, so the Hunk_Print () function
can display usage.

Hunk allocations are guaranteed to be 16 byte aligned.

The video buffers are allocated high to avoid leaving a hole underneath
server allocations when changing to a higher video mode.


Z_??? Zone memory functions used for small, dynamic allocations like text
strings from command input.  There is only about 48K for it, allocated at
the very bottom of the hunk.

Cache_??? Cache memory is for objects that can be dynamically loaded and
can usefully stay persistant between levels.  The size of the cache
fluctuates from level to level.

To allocate a cachable object


Temp_??? Temp memory is used for file loading and surface caching.  The size
of the cache memory is adjusted so that there is a minimum of 512k remaining
for temp memory.


------ Top of Memory -------

high hunk allocations

<--- high hunk reset point held by vid

video buffer

z buffer

surface cache

<--- high hunk used

cachable memory

<--- low hunk used

client and server low hunk allocations

<-- low hunk reset point held by host

startup hunk allocations

Zone block

----- Bottom of Memory -----



*/

use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null_mut;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CacheUserT {
    pub data: *mut c_void,
}

impl CacheUserT {
    pub const fn default() -> Self {
        Self { data: null_mut() }
    }
}

impl Default for CacheUserT {
    fn default() -> Self {
        Self::default()
    }
}

pub const DYNAMIC_SIZE: usize = 4 * 1024 * 1024;

pub const ZONEID: c_int = 0x1d4a11;
pub const MINFRAGMENT: usize = 64;

#[repr(C)]
pub struct MemBlockT {
    /// including the header and possibly tiny fragments
    pub size: c_int,
    /// a tag of 0 is a free block
    pub tag: c_int,
    /// should be ZONEID
    pub id: c_int,
    /// pad to 64 bit boundary
    pub pad: c_int,
    pub next: *mut MemBlockT,
    pub prev: *mut MemBlockT,
}

#[repr(C)]
pub struct MemZoneT {
    /// total bytes malloced, including header
    pub size: c_int,
    /// start / end cap for linked list
    pub blocklist: MemBlockT,
    pub rover: *mut MemBlockT,
}

/*
==============================================================================

                        ZONE MEMORY ALLOCATION

There is never any space between memblocks, and there will never be two
contiguous free memblocks.

The rover can be left pointing at a non-empty block

The zone calls are pretty much only used for small strings and structures,
all big things are allocated on the hunk.
==============================================================================
*/
#[no_mangle]
pub static mut mainzone: *mut MemZoneT = null_mut();

//============================================================================

pub const HUNK_SENTINEL: u32 = 0x1df001ed;

pub const HUNKNAME_LEN: usize = 24;

#[repr(C)]
pub struct HunkT {
    pub sentinel: c_int,
    /// including sizeof(hunk_t), -1 = not allocated
    pub size: c_int,
    pub name: [c_char; HUNKNAME_LEN],
}

/*
===============================================================================

CACHE MEMORY

===============================================================================
*/

pub const CACHENAME_LEN: usize = 32;
#[repr(C)]
pub struct CacheSystemT {
    // including this header
    pub size: c_int,
    pub user: *mut CacheUserT,
    pub name: [c_char; CACHENAME_LEN],
    pub prev: *mut CacheSystemT,
    pub next: *mut CacheSystemT,
    // for LRU flushing
    pub lru_prev: *mut CacheSystemT,
    pub lur_next: *mut CacheSystemT,
}

impl CacheSystemT {
    pub const fn default() -> Self {
        return Self {
            size: 0,
            user: null_mut(),
            name: [0; CACHENAME_LEN],
            prev: null_mut(),
            next: null_mut(),
            lru_prev: null_mut(),
            lur_next: null_mut(),
        };
    }
}

impl Default for CacheSystemT {
    fn default() -> Self {
        Self::default()
    }
}

#[allow(non_snake_case)]
pub mod capi {
    use std::mem::size_of;
    use std::os::raw::c_int;
    use std::ptr::null_mut;
    use zone::{CacheSystemT, MemBlockT, MemZoneT, ZONEID};
    use {Byte, QBoolean};

    #[no_mangle]
    pub static mut cache_head: CacheSystemT = CacheSystemT::default();

    #[no_mangle]
    pub static mut hunk_base: *mut Byte = null_mut();
    #[no_mangle]
    pub static mut hunk_size: c_int = 0;

    #[no_mangle]
    pub static mut hunk_low_used: c_int = 0;
    #[no_mangle]
    pub static mut hunk_high_used: c_int = 0;

    #[no_mangle]
    pub static mut hunk_tempactive: QBoolean = QBoolean::False;
    #[no_mangle]
    pub static mut hunk_tempmark: c_int = 0;

    #[no_mangle]
    pub unsafe fn Hunk_LowMark() -> c_int {
        return hunk_low_used;
    }

    //============================================================================
    #[no_mangle]
    pub unsafe fn Memory_InitZone(zone: *mut MemZoneT, size: c_int) {
        let block = (zone as *mut Byte).offset(size_of::<MemZoneT>() as isize) as *mut MemBlockT;

        // set the entire zone to one free block
        let mut z = &mut (*zone);
        z.blocklist.prev = block;
        z.blocklist.next = block;
        z.blocklist.tag = 1; // in use block
        z.blocklist.id = 0;
        z.blocklist.size = 0;
        z.rover = block;

        let mut b = &mut (*block);
        b.next = &mut z.blocklist;
        b.prev = &mut z.blocklist;
        b.tag = 0; // free block
        b.id = ZONEID;
        b.size = size - std::mem::size_of::<MemZoneT>() as c_int;
    }
}
