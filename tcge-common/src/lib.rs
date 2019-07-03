#[macro_use] extern crate log;
extern crate walkdir;

pub mod doublebuffer;

pub mod commandline;

pub mod gameloop;

pub mod resources;
pub use resources::Resources;
pub use resources::ResourceLocation;

pub fn current_time_nanos() -> u128 {
	use std::time::{SystemTime, UNIX_EPOCH};
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH)
		.expect("Time went backwards");
	return since_the_epoch.as_nanos();
}
