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
use crate::host::capi::host_parms;
use crate::{global_sdl_timer_context, isDedicated};
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::io::{Seek, SeekFrom};
use std::os::raw::c_char;
use std::sync::OnceLock;

thread_local! {
    // A crude abstraction to allow callers to operate on files without needing to manage underlying
    // native filesystem objects directly via a fixed-size array (to prevent runaway resource use)
    // of global system file handles. Use Sys_FileOpenRead/Write to get an index to the opened
    // handle, operate via the remaining Sys_File* interfaces, and then use Sys_FileClose to close
    // it freeing up an available slot.
    static SYS_HANDLES: std::cell::RefCell<[Option<File>; 30]> = const { std::cell::RefCell::new([const { None }; 30]) };
}

fn next_unused_handle() -> Result<usize, String> {
    SYS_HANDLES.with(|handles_cell| {
        let handles = handles_cell.borrow();
        for (idx, handle_opt) in handles.iter().enumerate() {
            if handle_opt.is_none() {
                return Ok(idx);
            }
        }
        Err("out of handles".to_string())
    })
}

fn remaining_filelength(file: &mut File) -> std::io::Result<u64> {
    let cur_pos = file.stream_position()?;
    let end_pos = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(cur_pos))?;
    Ok(end_pos - cur_pos)
}

static ERRORTXT1: &str = "\nERROR-OUT BEGIN\n\n";
static ERRORTXT2: &str = "\nQUAKE ERROR: ";

extern "C" {
    fn Host_Shutdown();
    fn PL_ErrorDialog(errorMsg: *const c_char);
}

#[cold]
pub fn sys_quit_with_error(err: impl Into<anyhow::Error>) -> ! {
    unsafe {
        (*host_parms).errstate += 1;
    }

    let is_dedicated = unsafe { isDedicated.as_bool() };

    let err = err.into();
    let text = err.to_string();
    if is_dedicated {
        print!("{}", text);
    }
    // SDL will put these into its own stderr log so print to stderr even in graphical mode.
    eprint!("{}", ERRORTXT1);
    unsafe {
        Host_Shutdown();
    }
    eprint!("{}{}\n\n", ERRORTXT2, text);

    if is_dedicated {
        print!("{}{}\r\n", ERRORTXT2, text);
        // Ensure console output is flushed so the user can see the error before sleep or exit.
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();

        global_sdl_timer_context.delay(3000); // show the console 3 more seconds
    } else {
        // Ensure console output is flushed before exiting and before showing the dialog.
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();

        // Showing the dialog is best-effort.
        if let Ok(ctext) = CString::new(text) {
            unsafe {
                PL_ErrorDialog(ctext.as_ptr());
            }
        }
    }

    std::process::exit(1);
}

pub fn sys_log_info(txt: &str) {
    print!("{}", txt);
}

pub fn sys_log_error(err: impl Into<anyhow::Error>) {
    let err = err.into();
    let text = err.to_string();

    eprint!("{}", text);
}

static CWD: OnceLock<CString> = OnceLock::new();

pub mod capi {
    use super::{next_unused_handle, remaining_filelength, sys_log_info, SYS_HANDLES};
    use super::{sys_quit_with_error, CWD};
    use crate::cvar::{CVarFlags, CVarT};
    use crate::host::capi::host_parms;
    use crate::{
        global_sdl_audio_context, global_sdl_context, global_sdl_controller_context,
        global_sdl_timer_context, global_sdl_video_context, QBoolean,
    };
    use std::env;
    use std::ffi::CString;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::raw::{c_char, c_double, c_int, c_ulong, c_void};
    use std::ptr::null_mut;
    use windows::Wdk::System::SystemServices::RtlGetVersion;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Media::timeBeginPeriod;
    use windows::Win32::System::Console::{AllocConsole, GetStdHandle};
    use windows::Win32::System::SystemInformation::{GetSystemInfo, OSVERSIONINFOEXW, SYSTEM_INFO};

    #[unsafe(no_mangle)]
    pub static mut isDedicated: QBoolean = QBoolean::False;

    #[unsafe(no_mangle)]
    pub static mut sys_throttle: CVarT = CVarT {
        name: b"sys_throttle\0".as_ptr() as *const c_char,
        string: b"0.02\0".as_ptr() as *const c_char,
        flags: CVarFlags::Archive,
        value: 0.02,
        default_string: b"0.02\0".as_ptr() as *const c_char,
        callback: None,
        next: null_mut(),
    }; // seconds

    #[unsafe(no_mangle)]
    pub static mut hinput: HANDLE = HANDLE(null_mut());
    #[unsafe(no_mangle)]
    pub static mut houtput: HANDLE = HANDLE(null_mut());

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_AtExit() {
        unsafe { sdl2::sys::SDL_Quit() };
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_InitSDL() {
        {
            let version = sdl2::version::version();
            sys_log_info(&format!(
                "Found SDL version {}.{}.{}\n",
                version.major, version.minor, version.patch
            ));
        }
        // N.B. quakespasm 0.96.3+ removed the SDL Version checks that were here. Currently, SDL
        // 2.26.x+ is required, but the latest stable 2.x version should work. SDL3 has since
        // replaced SDL2, but this project isn't ready for that yet.

        // Hints must be set *before* SDL is initialized.
        sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

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

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_FileOpenRead(path: *const c_char, hidx: *mut c_int) -> c_int {
        if hidx.is_null() {
            return -1;
        }
        unsafe { std::ptr::write(hidx, -1) }

        if path.is_null() {
            return -1;
        }

        let result = (|| -> Result<(c_int, c_int), ()> {
            let next_idx = next_unused_handle().map_err(|_| ())?;
            let fspath = unsafe { std::ffi::CStr::from_ptr(path) }
                .to_str()
                .map_err(|_| ())?;
            let mut f = File::open(fspath).map_err(|_| ())?;
            let len = remaining_filelength(&mut f).map_err(|_| ())? as c_int;

            SYS_HANDLES.with(|handles_cell| {
                handles_cell.borrow_mut()[next_idx] = Some(f);
            });

            Ok((next_idx as c_int, len))
        })();

        result.map_or(-1, |(open_idx, len)| {
            unsafe { std::ptr::write(hidx, open_idx) }
            len
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_FileOpenWrite(path: *const c_char) -> c_int {
        let result = (|| -> anyhow::Result<c_int> {
            anyhow::ensure!(
                !path.is_null(),
                "Unable to open file for write; no path specified"
            );
            let next_idx = next_unused_handle().map_err(|e| anyhow::anyhow!(e))?;

            let fspath = unsafe { std::ffi::CStr::from_ptr(path) }
                .to_str()
                .map_err(|e| anyhow::anyhow!(e))?;
            let f = File::create(fspath).map_err(|e| anyhow::anyhow!(e))?;

            SYS_HANDLES.with(|handles_cell| {
                handles_cell.borrow_mut()[next_idx] = Some(f);
            });

            Ok(next_idx as c_int)
        })();

        match result {
            Ok(hidx) => hidx,
            Err(err) => {
                sys_quit_with_error(anyhow::anyhow!(err));
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_FileClose(hidx: c_int) {
        (hidx >= 0).then(|| {
            SYS_HANDLES.with(|handles_cell| {
                // TODO: QuakeSpasm ignores failures here; should we fix that?
                handles_cell.borrow_mut()[hidx as usize].take();
            });
        });
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_FileSeek(hidx: c_int, pos: c_int) {
        (hidx >= 0 && pos >= 0).then(|| {
            SYS_HANDLES.with(|handles_cell| {
                let mut handles = handles_cell.borrow_mut();
                if let Some(f) = &mut handles[hidx as usize] {
                    // TODO: QuakeSpasm ignores failures here; fix it.
                    let _ = f.seek(SeekFrom::Start(pos as u64));
                }
            });
        });
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_FileRead(hidx: c_int, dest: *mut c_void, count: c_int) -> c_int {
        if hidx < 0 || dest.is_null() || count <= 0 {
            return 0;
        }

        SYS_HANDLES.with(|handles_cell| {
            let mut handles = handles_cell.borrow_mut();
            if let Some(f) = &mut handles[hidx as usize] {
                let buffer =
                    unsafe { std::slice::from_raw_parts_mut(dest as *mut u8, count as usize) };
                // TODO: QuakeSpasm ignores failures here; fix it.
                f.read(buffer).unwrap_or(0) as c_int
            } else {
                0
            }
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_FileWrite(hidx: c_int, data: *const c_void, count: c_int) -> c_int {
        if hidx < 0 || data.is_null() || count <= 0 {
            return 0;
        }

        SYS_HANDLES.with(|handles_cell| {
            let mut handles = handles_cell.borrow_mut();
            if let Some(f) = &mut handles[hidx as usize] {
                let buffer =
                    unsafe { std::slice::from_raw_parts_mut(data as *mut u8, count as usize) };
                // TODO: QuakeSpasm ignores failures here; fix it.
                f.write(buffer).unwrap_or(0) as c_int
            } else {
                0
            }
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_FileExists(path: *const c_char) -> c_int {
        if path.is_null() {
            // TODO: QuakeSpasm ignores failures here; fix it.
            return -1;
        }

        unsafe { std::ffi::CStr::from_ptr(path) }
            .to_str()
            .ok()
            .map_or(-1, |fspath| {
                if std::path::Path::new(fspath).is_file() {
                    1
                } else {
                    -1
                }
            })
    }

    // Historically, this would write the current directory to dst, but QuakeSpasm only ever called
    // it to initialize a global cwd variable once during Sys_Init() and never used it after. To
    // avoid unsafe code, this instead sets a global CWD that can be referred to directly by rust
    // callers safely allowing C callers to orchestrate the initialization but not the allocation or
    // lifetime.
    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_GetBaseDir(
        _argv0: *const c_char,
        _dst: *mut c_char,
        _dstsize: libc::size_t,
    ) {
        let result = (|| -> anyhow::Result<()> {
            let path = env::current_dir()?;
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("current directory path is not valid UTF-8"))?;
            let npath = if !path.has_root() || path.parent().is_some() {
                path_str.trim_end_matches(['/', '\\'])
            } else {
                path_str
            };
            let ncwd = CString::new(npath)?;
            CWD.set(ncwd)
                .map_err(|_| anyhow::anyhow!("CWD already set"))
        })();

        if let Err(err) = result {
            sys_quit_with_error(err);
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_Init() {
        // Set OS timer resolution to 1ms:
        // Works around buffer underruns with directsound and SDL2, but also will make Sleep() /
        // SDL_Delay() accurate to 1ms which should help framerate stability.
        unsafe {
            timeBeginPeriod(1);
        }

        Sys_GetBaseDir(null_mut(), null_mut(), 0); // Populate CWD
        match CWD.get() {
            Some(cwd) => unsafe {
                (*host_parms).basedir = cwd.as_ptr();
            },
            None => {
                sys_quit_with_error(anyhow::anyhow!("Couldn't determine basedir"));
            }
        };

        // userdirs not really necessary for windows guys;  can be done if necessary, though...
        unsafe {
            (*host_parms).userdir = (*host_parms).basedir; // code elsewhere relies on this !
        }

        let mut osvi = OSVERSIONINFOEXW {
            dwOSVersionInfoSize: size_of::<OSVERSIONINFOEXW>() as u32,
            ..Default::default()
        };

        unsafe {
            if RtlGetVersion(&mut osvi as *mut _ as *mut _).is_err() {
                sys_quit_with_error(anyhow::anyhow!("Couldn't get OS info"));
            }
        }

        if (osvi.dwMajorVersion < 4)
            || (osvi.dwPlatformId
                == windows::Win32::System::Diagnostics::Debug::VER_PLATFORM_WIN32s.0)
        {
            sys_quit_with_error(anyhow::anyhow!(
                "QuakeSpasm requires at least Win95 or NT 4.0"
            ));
        }

        if osvi.dwPlatformId == windows::Win32::System::Diagnostics::Debug::VER_PLATFORM_WIN32_NT.0
        {
            let mut ossi = SYSTEM_INFO {
                ..Default::default()
            };
            unsafe {
                GetSystemInfo(&mut ossi as *mut _ as *mut _);
                (*host_parms).numcpus = if ossi.dwNumberOfProcessors >= 1 {
                    ossi.dwNumberOfProcessors as c_int
                } else {
                    1
                };
            };
        } else {
            // Win95: Win9x or WinME
            unsafe {
                (*host_parms).numcpus = 1;
            };
        }
        sys_log_info(&format!("Detected {} CPU(s).\n", unsafe {
            (*host_parms).numcpus
        }));

        unsafe {
            if isDedicated.as_bool() {
                if let Err(err) = AllocConsole() {
                    isDedicated = QBoolean::False; // ensure graphical error dialog exists
                    sys_quit_with_error(anyhow::anyhow!(
                        "Couldn't create dedicated server console: {}",
                        err
                    ));
                }

                if let Ok(h) = GetStdHandle(windows::Win32::System::Console::STD_INPUT_HANDLE) {
                    hinput = h;
                } else {
                    sys_quit_with_error(anyhow::anyhow!(
                        "Couldn't initialize server console stdin"
                    ));
                }

                if let Ok(h) = GetStdHandle(windows::Win32::System::Console::STD_OUTPUT_HANDLE) {
                    houtput = h;
                } else {
                    sys_quit_with_error(anyhow::anyhow!(
                        "Couldn't intialize server console stdout"
                    ));
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_mkdir(path: *const c_char) {
        let result = (|| -> anyhow::Result<()> {
            anyhow::ensure!(
                !path.is_null(),
                "Unable to create directory; no path specified"
            );
            let fspath = unsafe { std::ffi::CStr::from_ptr(path) }.to_str()?;
            std::fs::create_dir(fspath).or_else(|e| {
                if matches!(e.kind(), std::io::ErrorKind::AlreadyExists) {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
            Ok(())
        })();

        if let Err(err) = result {
            sys_quit_with_error(err);
        }
    }

    extern "C" {
        fn Host_Shutdown();
        fn FreeConsole();
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_Quit() {
        unsafe {
            Host_Shutdown();

            if isDedicated.as_bool() {
                FreeConsole();
            }
        }
        std::process::exit(0);
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_DoubleTime() -> c_double {
        (global_sdl_timer_context.ticks() as c_double) / 1000.0
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_Sleep(msecs: c_ulong) {
        // TODO: Replace with std::thread::sleep
        global_sdl_timer_context.delay(msecs)
    }

    extern "C" {
        fn IN_Commands();
        fn IN_SendKeyEvents();
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Sys_SendKeyEvents() {
        unsafe {
            IN_Commands(); //ericw -- allow joysticks to add keys so they can be used to confirm SCR_ModalMessage
            IN_SendKeyEvents();
        }
    }
}
