/*
 * net_defs.h -- functions and data private to the network layer
 * net_sys.h and its dependencies must be included before net_defs.h.
 *
 * Copyright (C) 1996-1997  Id Software, Inc.
 * Copyright (C) 2005-2012  O.Sezer <sezero@users.sourceforge.net>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or (at
 * your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

use net::{NET_MAXMESSAGE, NET_NAMELEN};
use net_sys::SysSocketT;
use std::mem::size_of;
use std::os::raw::{c_char, c_double, c_int, c_short, c_uchar, c_uint, c_void};
use Byte;
use {QBoolean, MAX_DATAGRAM};

#[repr(C)]
pub struct QSockAddr {
    /*
    #if defined(HAVE_SA_LEN)
    unsigned char qsa_len;
    unsigned char qsa_family;
    #else
    */
    qsa_family: c_short,

    // #endif	/* BSD, sockaddr */
    qsa_data: [c_uchar; 14],
}

pub const NET_HEADERSIZE: usize = 2 * size_of::<c_uint>();
pub const NET_DATAGRAMSIZE: usize = MAX_DATAGRAM + NET_HEADERSIZE;

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct NetHeaderFlags: c_uint {
        const LengthMask = 0x0000ffff;
        const Data = 0x00010000;
        const Ack = 0x00020000;
        const Nak = 0x00040000;
        const Eom = 0x00080000;
        const Unreliable = 0x00100000;
        const Ctl = 0x80000000;
    }
}

// #if (NETFLAG_LENGTH_MASK & NET_MAXMESSAGE) != NET_MAXMESSAGE
// #error "NET_MAXMESSAGE must fit within NETFLAG_LENGTH_MASK"
// #endif

pub const NET_PROTOCOL_VERSION: u32 = 3;

/**

This is the network info/connection protocol.  It is used to find Quake
servers, get info about them, and connect to them.  Once connected, the
Quake game protocol (documented elsewhere) is used.


General notes:
    game_name is currently always "QUAKE", but is there so this same protocol
        can be used for future games as well; can you say Quake2?

CCREQ_CONNECT
        string	game_name		"QUAKE"
        byte	net_protocol_version	NET_PROTOCOL_VERSION

CCREQ_SERVER_INFO
        string	game_name		"QUAKE"
        byte	net_protocol_version	NET_PROTOCOL_VERSION

CCREQ_PLAYER_INFO
        byte	player_number

CCREQ_RULE_INFO
        string	rule

CCREP_ACCEPT
        long	port

CCREP_REJECT
        string	reason

CCREP_SERVER_INFO
        string	server_address
        string	host_name
        string	level_name
        byte	current_players
        byte	max_players
        byte	protocol_version	NET_PROTOCOL_VERSION

CCREP_PLAYER_INFO
        byte	player_number
        string	name
        long	colors
        long	frags
        long	connect_time
        string	address

CCREP_RULE_INFO
        string	rule
        string	value

    note:
        There are two address forms used above.  The short form is just a
        port number.  The address that goes along with the port is defined as
        "whatever address you receive this reponse from".  This lets us use
        the host OS to solve the problem of multiple host addresses (possibly
        with no routing between them); the host will use the right address
        when we reply to the inbound connection request.  The long from is
        a full address and port in a string.  It is used for returning the
        address of a server that is not running locally.

**/

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum ClientConnectMessage {
    ReqConnect = 0x01,
    ReqServerInfo = 0x02,
    ReqPlayerInfo = 0x03,
    ReqRuleInfo = 0x04,

    RepAccept = 0x81,
    RepReject = 0x82,
    RepServerInfo = 0x83,
    RepPlayerInfo = 0x84,
    RepRuleInfo = 0x85,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct QSocketT {
    pub next: *mut QSocketT,
    pub connecttime: c_double,
    pub lastMessageTime: c_double,
    pub lastSendTime: c_double,

    pub disconnected: QBoolean,
    pub canSend: QBoolean,
    pub sendNext: QBoolean,

    pub driver: c_int,
    pub landriver: c_int,
    pub socket: SysSocketT,
    pub driverdata: *mut c_void,

    pub ackSequence: c_uint,
    pub sendSequence: c_uint,
    pub unreliableSendSequence: c_uint,
    pub sendMessageLength: c_int,
    pub sendMessage: [Byte; NET_MAXMESSAGE],

    pub receiveSequence: c_uint,
    pub unreliableReceiveSequence: c_uint,
    pub receiveMessageLength: c_int,
    pub receiveMessage: [Byte; NET_MAXMESSAGE],

    pub addr: QSockAddr,
    pub address: [c_char; NET_NAMELEN],
}
