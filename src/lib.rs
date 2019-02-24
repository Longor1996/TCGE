#[macro_use] extern crate failure;
pub const MAGIC : u32 = 42;

pub mod resources;
pub mod gameloop;
pub mod blocks;
pub mod client;
pub mod server;
pub mod util;
