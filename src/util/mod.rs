//! A collection of small self-contained modules that do exactly one thing.

pub mod utf8;
pub mod gameloop;

pub fn current_time_nanos() -> u128 {
	use std::time::{SystemTime, UNIX_EPOCH};
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH)
		.expect("Time went backwards");
	return since_the_epoch.as_nanos();
}
