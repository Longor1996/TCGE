#[macro_use] extern crate log;
extern crate walkdir;
extern crate time;

pub mod doublebuffer;

pub mod commandline;

pub mod gameloop;

pub mod profiler;

pub mod resources;
pub use resources::Resources;
pub use resources::ResourceLocation;

pub fn current_time_nanos() -> u128 {
	use std::time::{SystemTime, UNIX_EPOCH};
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH)
		.expect("Time went backwards");
	since_the_epoch.as_nanos()
}

pub fn current_time_nanos_precise() -> u64 {
	time::precise_time_ns()
}
