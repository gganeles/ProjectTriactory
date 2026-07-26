//! Lightyear client: connects to `triactory_shared::config::DEV_SERVER_ADDR` over UDP with a
//! locally-generated netcode.io connect token (no auth service exists yet — see
//! `server/src/netcode.rs`'s matching caveat).
//!
//! Connects once at `Startup`, then [`reconnect_on_disconnect`] retries a fixed delay after any
//! disconnect (e.g. the dev server restarting) — full reconnect semantics (backoff,
//! app-resume-on-mobile) are milestone M10, this is just enough for local dev.

use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
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

/// How long to wait after a disconnect before trying again.
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

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

/// Despawns the (now-dead) client entity on disconnect and retries a fresh
/// [`connect_to_server`] after [`RECONNECT_DELAY`] — covers the server restarting (e.g. during
/// local dev) rather than leaving the client permanently given up.
pub fn reconnect_on_disconnect(
    mut commands: Commands,
    disconnected: Query<Entity, (With<Client>, Added<Disconnected>)>,
    mut pending: Local<Option<Timer>>,
    time: Res<Time>,
) -> Result {
    for entity in &disconnected {
        commands.entity(entity).despawn();
        *pending = Some(Timer::new(RECONNECT_DELAY, TimerMode::Once));
        info!("Lost connection to server; retrying in {RECONNECT_DELAY:?}");
    }

    if let Some(timer) = pending.as_mut()
        && timer.tick(time.delta()).just_finished()
    {
        *pending = None;
        connect_to_server(commands)?;
    }
    Ok(())
}
