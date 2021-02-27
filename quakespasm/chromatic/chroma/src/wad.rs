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
use std::os::raw::{c_char, c_int};
use std::ptr::null_mut;
use Byte;
use {wad_lumps, wad_numlumps, W_CleanupName};

/// LumpinfoT.compression types:
pub const CMP_NONE: u32 = 0;
pub const CMP_LZSS: u32 = 1;

/// LumpinfoT.r#type types:
pub const TYP_NONE: u32 = 0;
pub const TYP_LABEL: u32 = 1;
pub const TYP_LUMPY: u32 = 64;
/// 64 + grab command number
pub const TYP_PALETTE: u32 = 64;
pub const TYP_QTEX: u32 = 65;
pub const TYP_QPIC: u32 = 66;
pub const TYP_SOUND: u32 = 67;
pub const TYP_MIPTEX: u32 = 68;

/// Common constants:
pub const WADFILENAME: &'static [u8] = b"gfx.wad\0";

#[derive(Clone, Copy)]
#[repr(C)]
pub struct QPicT {
    pub width: c_int,
    pub height: c_int,
    pub data: [Byte; 0],
}

#[repr(C)]
pub struct WadinfoT {
    /// should be WAD2 or 2DAW
    pub identification: [c_char; 4],
    pub numlumps: c_int,
    pub infotableofs: c_int,
}

#[repr(C)]
pub struct LumpinfoT {
    pub filepos: c_int,
    pub disksize: c_int,
    pub size: c_int,
    pub r#type: c_char,
    pub compression: c_char,
    pub pad1: c_char,
    pub pad2: c_char,
    pub name: [c_char; 16],
}

unsafe fn w_get_lumpinfo(name: *const c_char) -> *mut LumpinfoT {
    let mut clean: [c_char; 16] = Default::default();

    W_CleanupName(name, clean.as_mut_ptr());

    let numlumps = wad_numlumps;
    let mut lump_p: *mut LumpinfoT = wad_lumps.as_mut().unwrap();

    for _ in 0..numlumps {
        if libc::strcmp(clean.as_ptr(), (*lump_p).name.as_mut_ptr()) == 0 {
            return lump_p;
        }

        lump_p = lump_p.add(1);
    }

    //Con_SafePrintf ("w_get_lumpinfo: %s not found\n", name); //johnfitz -- was Sys_Error

    null_mut()
}

pub mod capi {
    use std::os::raw::{c_char, c_int, c_void};
    use std::ptr::null_mut;
    use std::slice;
    use wad::{w_get_lumpinfo, LumpinfoT, QPicT};
    use Byte;

    #[no_mangle]
    pub static mut wad_numlumps: c_int = 0;

    #[no_mangle]
    pub static mut wad_lumps: *mut LumpinfoT = null_mut();

    #[no_mangle]
    pub static mut wad_base: *mut Byte = null_mut();

    /// Lowercases name and pads with spaces and a terminating 0 to the length of
    /// LumpinfoT->name.
    ///
    /// Used so lumpname lookups can proceed rapidly by comparing 4 chars at a time
    /// Space padding is so names can be printed nicely in tables.
    /// Can safely be performed in place.
    ///
    #[no_mangle]
    pub unsafe extern "C" fn W_CleanupName(inn: *const c_char, outn: *mut c_char) {
        let in_slice = slice::from_raw_parts(inn as *const u8, 16);
        let out_slice = slice::from_raw_parts_mut(outn as *mut u8, 16);

        for (idx, c) in in_slice.iter().enumerate() {
            match *c {
                0 => {
                    out_slice[idx..].fill(0);
                    break;
                }
                b'A'..=b'Z' => {
                    out_slice[idx] = *c + (b'a' - b'A');
                }
                _ => out_slice[idx] = *c,
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn W_GetLumpName(name: *const c_char) -> *mut c_void {
        let lump_p = w_get_lumpinfo(name);

        if lump_p != null_mut() {
            let filepos = (&(*lump_p)).filepos as isize;
            let lump_p = wad_base.offset(filepos);
            return lump_p as *mut c_void;
        }

        return null_mut();
    }

    #[no_mangle]
    pub unsafe extern "C" fn SwapPic(pic: *mut QPicT) {
        (&mut *pic).width = (&*pic).width.to_le();
        (&mut *pic).height = (&*pic).height.to_le();
    }

    /*
    extern "C" {
        fn COM_LoadMallocFile(path: *const c_char, path_id: *const c_uint) -> *mut Byte;
        fn Sys_Error(error: *const c_char, ...);

        pub static mut com_basedir: [c_char; MAX_OSPATH as usize];
    }

    #[no_mangle]
    pub unsafe extern "C" fn W_LoadWadFile()
    {
        //TODO: use cache_alloc
        if !wad_base.is_null()
        {
            libc::free(wad_base as *mut c_void);
        }

        let filename = WADFILENAME.as_ptr() as *const c_char;
        wad_base = COM_LoadMallocFile(filename, null());

        if wad_base.is_null()
        {
            Sys_Error(b"W_LoadWadFile: couldn't load %s\n\n \
                Basedir is: %s\n\n \
                Check that this has an %s subdirectory containing pak0.pak and pak1.pak, \
                or use the -basedir command-line option to specify another directory.\0".as_ptr() as *const c_char,
                      filename, com_basedir.as_ptr(), GAMENAME.as_ptr());
        }

        let header = &*(wad_base as *mut WadinfoT);

        if header.identification[0] != 'W' as c_char || header.identification[1] != 'A' as c_char
            || header.identification[2] != 'D' as c_char || header.identification[3] != '2' as c_char
        {
            Sys_Error(b"Wad file %s doesn't have WAD2 id\n\0".as_ptr() as *const c_char, filename);
        }

        wad_numlumps = header.numlumps.to_le();

        let infotableofs = header.infotableofs.to_le() as isize;
        wad_lumps = wad_base.offset(infotableofs) as *mut LumpinfoT;

        let mut lump_p: *mut LumpinfoT = wad_lumps.as_mut().unwrap();
        for _ in 0..wad_numlumps {
            let lump = &mut *lump_p;
            lump.filepos = lump.filepos.to_le();
            lump.size = lump.size.to_le();

            W_CleanupName(lump.name.as_ptr(), lump.name.as_mut_ptr()); // CAUTION: in-place editing!!!

            if lump.r#type == TYP_QPIC as c_char
            {
                SwapPic(wad_base.offset(lump.filepos as isize) as *mut QpicT);
            }

            lump_p = lump_p.add(1);
        }
    }
    */
}
