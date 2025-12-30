/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2002-2005 John Fitzgibbons and others
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

pub mod capi {
    use crate::{
        global_sdl_audio_context, global_sdl_context, global_sdl_controller_context,
        global_sdl_timer_context, global_sdl_video_context,
    };
    use std::os::raw::c_double;

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_DoubleTime() -> c_double {
        return (global_sdl_timer_context.ticks() as c_double) / 1000.0;
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_AtExit() {
        // IOU: Attempts to use global sdl_context objects will fail after this. Need a cleaner way
        // to manage globals.
        unsafe { sdl2::sys::SDL_Quit() };
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_InitSDL() {
        //let sdl_version = sdl2::version::version();

        //Sys_Printf("Found SDL version %i.%i.%i\n",sdl_version->major,sdl_version->minor,sdl_version->patch);
        // N.B. quakespasm 0.96.3+ removed the SDL Version checks that were here. Currently, SDL
        // 2.26.x+ is required, but the latest stable 2.x version should work. SDL3 has since
        // replaced SDL2, but this project isn't ready for that yet.

        // Global lazy statics are initialized on first deref; do so in required order.
        let _init_sdl_context = &*global_sdl_context;
        let _init_sdl_audio_context = &*global_sdl_audio_context;
        let _init_sdl_controller_context = &*global_sdl_controller_context;
        let _init_sdl_timer_context = &*global_sdl_timer_context;
        let _init_sdl_video_context = &*global_sdl_video_context;

        // Normally drop() would be invoked on the global_sdl_context
        unsafe {
            libc::atexit(Sys_AtExit);
        }
    }
}
