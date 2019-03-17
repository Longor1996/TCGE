#[macro_use] extern crate failure;
#[macro_use] extern crate mopa;
extern crate core;

pub const MAGIC : u32 = 42;

pub mod resources;
pub mod gameloop;
pub mod router;
pub mod blocks;
pub mod client;
pub mod server;
pub mod util;
