//! Lightyear client: connects to `triactory_shared::config::DEV_SERVER_ADDR` over UDP with a
//! locally-generated netcode.io connect token (no auth service exists yet — see
//! `server/src/netcode.rs`'s matching caveat).
//!
//! Connects once at `Startup`. Mobile OSes suspend sockets on background — treating app-resume
//! as a reconnect is a later concern (milestone M10), not handled yet.

use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use triactory_shared::config::{DEV_SERVER_ADDR, PROTOCOL_ID};

/// An OS-assigned ephemeral port rather than a fixed one, so multiple clients can run on the same
/// machine during development without port conflicts.
const CLIENT_LOCAL_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

/// No real player identity yet (no login/lobby), so every client claims id 0. Fine while only
/// one client connects at a time during development; needs real per-player ids by milestone M9.
const DEV_CLIENT_ID: u64 = 0;

pub fn connect_to_server(mut commands: Commands) -> Result {
    let auth = Authentication::Manual {
        server_addr: DEV_SERVER_ADDR,
        client_id: DEV_CLIENT_ID,
        private_key: Key::default(),
        protocol_id: PROTOCOL_ID,
    };
    let client = commands
        .spawn((
            Client::default(),
            LocalAddr(CLIENT_LOCAL_ADDR),
            PeerAddr(DEV_SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver,
            NetcodeClient::new(auth, NetcodeConfig::default())?,
            UdpIo::default(),
        ))
        .id();
    commands.trigger(Connect { entity: client });
    Ok(())
}
