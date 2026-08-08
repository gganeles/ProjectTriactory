//! Tuning constants shared by both apps.

use core::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Default hexagon edge length (triangles per side) used until a real lobby lets players
/// choose a map size.
pub const DEFAULT_EDGE_TILES: i32 = 31;

/// Minimum/maximum players a match supports. A real lobby will enforce this before match start;
/// map generation (`server/src/game/map`) re-validates it defensively.
pub const MIN_PLAYERS: u8 = 2;
pub const MAX_PLAYERS: u8 = 6;

/// Fixed-timestep simulation rate shared by both apps (see the root README's locked decisions).
pub const TICK_RATE_HZ: f64 = 30.0;

/// Identifies this app's wire protocol to netcode.io (the ASCII bytes of "TRIA"), so a client
/// won't accidentally connect to an unrelated server speaking the same transport.
pub const PROTOCOL_ID: u64 = 0x5452_4941;

/// UDP port the server listens on.
pub const SERVER_PORT: u16 = 5000;

/// Where the client connects for local development — the server also binds to this exact
/// address. A real deployment needs a config/discovery step instead of a hardcoded loopback
/// address; that's milestone M9/M10 (match lifecycle/lobby, mobile builds), not yet in scope.
pub const DEV_SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);
