//! Lightyear server setup: UDP transport + netcode.io auth, listening on
//! `triactory_shared::config::DEV_SERVER_ADDR`.
//!
//! Connect tokens are validated against a private key that's just `Key::default()` (all-zero
//! bytes, the netcode default) — fine for local development since there's no real auth service
//! yet. See `DEV_SERVER_ADDR`'s doc comment for the matching caveat on the address.

use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use triactory_shared::config::{DEV_SERVER_ADDR, PROTOCOL_ID};

pub fn start_server(mut commands: Commands) {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default().with_protocol_id(PROTOCOL_ID)),
            LocalAddr(DEV_SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();
    commands.trigger(Start { entity: server });
    info!("Listening on {DEV_SERVER_ADDR} (protocol id {PROTOCOL_ID})");
}
