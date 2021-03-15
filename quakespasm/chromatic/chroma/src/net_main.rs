/*
Copyright (C) 1996-2001 Id Software, Inc.
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
    use std::os::raw::{c_char, c_double, c_int};
    use std::ptr::null_mut;
    use net::NET_NAMELEN;
    use net_defs::QSocketT;
    use ::{SizeBufT, QBoolean};
    use cvar::{CVarT, CVarFlags};

    #[no_mangle]
    pub static mut net_activeSockets: *mut QSocketT = null_mut();

    #[no_mangle]
    pub static mut net_freeSockets: *mut QSocketT = null_mut();
    #[no_mangle]
    pub static mut net_numsockets: c_int = 0;

    #[no_mangle]
    pub static mut ipxAvailable: QBoolean = QBoolean::False;
    #[no_mangle]
    pub static mut tcpipAvailable: QBoolean = QBoolean::False;

    #[no_mangle]
    pub static mut net_hostport: c_int = 0;
    #[no_mangle]
    pub static mut DEFAULTnet_hostport: c_int = 26000;

    #[no_mangle]
    pub static mut my_ipx_address: [c_char; NET_NAMELEN] = [0; NET_NAMELEN];
    #[no_mangle]
    pub static mut my_tcpip_address: [c_char; NET_NAMELEN] = [0; NET_NAMELEN];

    // IOU: Export for now, but make this private once conversion is complete.
    #[no_mangle]
    pub static mut listening: QBoolean = QBoolean::False;

    #[no_mangle]
    pub static mut slistInProgress: QBoolean = QBoolean::False;
    #[no_mangle]
    pub static mut slistSilent: QBoolean = QBoolean::False;
    #[no_mangle]
    pub static mut slistLocal: QBoolean = QBoolean::True;
    // IOU: Export for now, but make this private once conversion is complete.
    #[no_mangle]
    pub static mut slistStartTime: c_double = 0.0;
    // IOU: Export for now, but make this private once conversion is complete.
    #[no_mangle]
    pub static mut slistLastShown: c_int = 0;

    /*
    static void Slist_Send (void *);
    static void Slist_Poll (void *);
    // IOU: Export for now, but make this private once conversion is complete.
    #[no_mangle]
    static PollProcedure	slistSendProcedure = {NULL, 0.0, Slist_Send};
    // IOU: Export for now, but make this private once conversion is complete.
    #[no_mangle]
    static PollProcedure	slistPollProcedure = {NULL, 0.0, Slist_Poll};
    */

    #[no_mangle]
    pub static mut net_message: SizeBufT = SizeBufT::default();
    #[no_mangle]
    pub static mut net_activeconnections: c_int = 0;

    #[no_mangle]
    pub static mut messagesSent: c_int = 0;
    #[no_mangle]
    pub static mut messagesReceived: c_int = 0;
    #[no_mangle]
    pub static mut unreliableMessagesSent: c_int = 0;
    #[no_mangle]
    pub static mut unreliableMessagesReceived: c_int = 0;

    // IOU: Export for now, but make this private once conversion is complete.
    #[no_mangle]
    pub static mut net_messagetimeout: CVarT = CVarT {
        name: b"net_messagetimeout\0".as_ptr() as *const c_char,
        string: b"300\0".as_ptr() as *const c_char,
        flags: CVarFlags::None,
        value: 0.0,
        default_string: b"300\0".as_ptr() as *const c_char,
        callback: None,
        next: std::ptr::null_mut(),
    };
    #[no_mangle]
    pub static mut hostname: CVarT = CVarT {
        name: b"hostname\0".as_ptr() as *const c_char,
        string: b"UNNAMED\0".as_ptr() as *const c_char,
        flags: CVarFlags::None,
        value: 0.0,
        default_string: b"UNNAMED\0".as_ptr() as *const c_char,
        callback: None,
        next: std::ptr::null_mut(),
    };

    #[no_mangle]
    pub static mut net_driverlevel: c_int = 0;

    #[no_mangle]
    pub static mut net_time: c_double = 0.0;

    /*
    #[no_mangle]
    pub extern "C" fn SetNetTime() -> c_double
    {
        unsafe {
            net_time = Sys_DoubleTime();
            return net_time;
        }
    }
    */
}
