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
// cvar.rs -- dynamic variable tracking

/*
cvar_t variables are used to hold scalar or string variables that can
be changed or displayed at the console or prog code as well as accessed
directly in C code.

it is sufficient to initialize a cvar_t with just the first two fields,
or you can add a ,true flag for variables that you want saved to the
configuration file when the game is quit:

cvar_t	r_draworder = {"r_draworder","1"};
cvar_t	scr_screensize = {"screensize","1",true};

Cvars must be registered before use, or they will have a 0 value instead
of the float interpretation of the string.
Generally, all cvar_t declarations should be registered in the apropriate
init function before any console commands are executed:

Cvar_RegisterVariable (&host_framerate);


C code usually just references a cvar in place:
if ( r_draworder.value )

It could optionally ask for the value to be looked up for a string name:
if (Cvar_VariableValue ("r_draworder"))

Interpreted prog code can access cvars with the cvar(name) or
cvar_set (name, value) internal functions:
teamplay = cvar("teamplay");
cvar_set ("registered", "1");

The user can access cvars from the console in two ways:
r_draworder		prints the current value
r_draworder 0		sets the current value to 0

Cvars are restricted from having the same names as commands to keep this
interface from being ambiguous.

*/

use std::os::raw::{c_char, c_float, c_uint};

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct CVarFlags: c_uint {
        const None = 0;
        /// causes it to be saved to config
        const Archive = 1 << 0;
        /// changes will be broadcasted to all players (q1)
        const Notify = 1 << 1;
        /// added to serverinfo will be sent to clients (q1/net_dgrm.c and qwsv)
        const ServerInfo = 1 << 2;
        /// added to userinfo; will be sent to server (qwcl)
        const UserInfo = 1 << 3;
        const Changed = 1 << 4;
        const Rom = 1 << 6;
        /// locked temporarily
        const Locked = 1 << 8;
        /// the var is added to the list of variables
        const Registered = 1 << 10;
        /// var has a callback
        const Callback = 1 << 16;
    }
}

pub type CVarCallbackT = ::std::option::Option<unsafe extern "C" fn(arg1: *mut CVarT)>;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CVarT {
    pub name: *const c_char,
    pub string: *const c_char,
    pub flags: CVarFlags,
    pub value: c_float,
    /// remember defaults for reset function
    pub default_string: *const c_char,
    pub callback: CVarCallbackT,
    pub next: *mut CVarT,
}

#[allow(bad_style)]
pub mod capi {
    use super::{CVarCallbackT, CVarFlags, CVarT};
    use libc::{fileno, get_osfhandle, FILE};
    use std::os::windows::io::FromRawHandle;
    use std::os::windows::raw::HANDLE;
    use std::{
        ffi::CStr,
        fs::File,
        io::Write,
        os::raw::{c_char, c_float},
        ptr::null_mut,
    };

    #[no_mangle]
    pub static mut cvar_vars: *mut CVarT = null_mut();
    #[no_mangle]
    pub static cvar_null_string: &'static [u8] = b"\0";

    //==============================================================================
    //
    //  CVAR FUNCTIONS
    //
    //==============================================================================

    #[no_mangle]
    pub unsafe fn Cvar_FindVar(var_name: *const c_char) -> *mut CVarT {
        let find_str = CStr::from_ptr(var_name);

        let mut var = cvar_vars;
        while !var.is_null() {
            let name_str = CStr::from_ptr((&*var).name);
            if name_str == find_str {
                return var;
            }

            var = (&*var).next;
        }

        return null_mut();
    }

    #[no_mangle]
    pub unsafe fn Cvar_FindVarAfter(prev_name: *const c_char, with_flags: CVarFlags) -> *mut CVarT {
        let mut var;

        if !prev_name.is_null() && *prev_name != 0 {
            var = Cvar_FindVar(prev_name);
            if var.is_null() {
                return null_mut();
            }

            var = (&*var).next;
        } else {
            var = cvar_vars;
        }

        if with_flags.is_empty() {
            return var;
        }

        // search for the next cvar matching the needed flags
        while !var.is_null() {
            if (&*var).flags.contains(with_flags) {
                return var;
            }

            var = (&*var).next;
        }

        return null_mut();
    }

    #[no_mangle]
    pub unsafe fn Cvar_VariableString(var_name: *const c_char) -> *const c_char {
        if let Some(var) = Cvar_FindVar(var_name).as_ref() {
            return (&*var).string;
        }
        return cvar_null_string.as_ptr() as *const c_char;
    }

    #[no_mangle]
    pub unsafe fn Cvar_VariableValue(var_name: *const c_char) -> c_float {
        if let Some(var) = Cvar_FindVar(var_name).as_ref() {
            return crate::Q_atof((&*var).string);
        }
        return 0.0;
    }

    /*
    ============
    Cvar_SetCallback

    Set a callback function to the var
    ============
    */
    #[no_mangle]
    pub unsafe fn Cvar_SetCallback(var: *mut CVarT, func: CVarCallbackT) {
        (&mut *var).callback = func;
        if func.is_some() {
            (&mut *var).flags |= CVarFlags::Callback;
        } else {
            (&mut *var).flags &= !CVarFlags::Callback;
        }
    }

    /*
    ============
    Cvar_WriteVariables

    Writes lines containing "set variable value" for all variables
    with the archive flag set to true.
    ============
    */
    #[no_mangle]
    pub unsafe fn Cvar_WriteVariables(f: *mut FILE) {
        let fd = fileno(f);
        let h = get_osfhandle(fd) as HANDLE;
        let mut file = File::from_raw_handle(h);

        let mut var = cvar_vars;
        while !var.is_null() {
            if (&*var).flags.contains(CVarFlags::Archive) {
                let name = CStr::from_ptr((&*var).name);
                let value = CStr::from_ptr((&*var).string);

                if let Err(_e) = writeln!(
                    file,
                    r#"{} "{}""#,
                    name.to_str().unwrap_or(""),
                    value.to_str().unwrap_or("")
                ) {
                    eprintln!("Cvar_WriteVariables failed: {}!", _e);
                    return;
                }
            }

            var = (&*var).next;
        }
    }
}
