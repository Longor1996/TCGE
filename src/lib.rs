//! The **Talecraft Game Engine** for discrete voxels (blocks!).
//!
//! > Notice: **This game-engine is not associated with *Mojang AB* in any way**.
//! >
//! > The name 'talecraft' will be changed once a better name is found.
//!
//! A discrete voxel is also commonly called *Block*, *Bloxel* or *Cubic Voxel*.
//!
//! # Architecture
//!
//! The engine is split into three general parts:
//! - The `client`-module and associated binary contains all code for the game-client.
//! - The `server`-module and associated binary contains all code for the game-server.
//! - All other modules define the 'core' of the engine and the actual 'logic',
//!   and have no knowledge of what a client or server even is.
//!
//! All the while, the client does not know what a server is, and vice-versa.
//!
//! ## Routing / Backbone
//!
//! Both the client and the server make use of the `router` module to abstract
//! their structure, so that (global) state-keeping is reduced and concentrated
//! into the various components attached to the router.
//!
//! An interesting side-effect of this is that application-state can be
//! 'linked to', by writing out the path of the lens in the routing tree,
//! applying generic filesystem/URL semantics to it and using that to move lenses around.
//!
//! This functionality is exposed in the client as a commandline argument.

#[macro_use] extern crate log;
#[macro_use] extern crate failure;
#[macro_use] extern crate mopa;
extern crate core;

pub mod resources;
pub mod router;
pub mod blocks;
pub mod client;
pub mod server;
pub mod util;
