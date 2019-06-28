/// Creates a new gameloop with the given number of ticks per second.
pub fn new(ticks_per_second: i32) -> State {
	State::new(ticks_per_second)
}

/// Represents the continuous state of a gameloop.
pub struct State {
	/// The amount of ticks to attempt per second.
	ticks_per_second: i32,
	
	/// The normal duration between individual ticks.
	skip_ticks: f64,
	
	/// How many frames can be skipped when catching up with lost ticks.
	max_frameskip: i32,
	
	/// The moment when the next tick happens.
	next_game_tick: f64,
	
	/// Counter of catch-up ticks per frame.
	loops: i32,
	
	/// The measured length in seconds of the last frame on the CPU side.
	frame_time: f64,
	
	/// The x-per-second counter for frames.
	frame_count: usize,
	
	/// The x-per-second counter for ticks.
	tick_count: i32,
	
	/// Total number of frames the gameloop went trough.
	total_frames: u64,
	
	/// Total number of ticks the gameloop went trough.
	total_ticks: u64,
	
	/// The time when the last x-per-second measurements where done.
	last_chk: f64,
	
	state: LoopState,
}

impl State {
	/// Creates a new gameloop with the given ticks-per-second as approximate goal.
	pub fn new(ticks_per_second: i32) -> Self {
		Self {
			ticks_per_second,
			skip_ticks: 1.0 / (ticks_per_second as f64),
			max_frameskip: 5,
			loops: 0,
			tick_count: 0,
			next_game_tick: 0.0,
			frame_time: 0.0,
			frame_count: 0,
			last_chk: 0.0,
			total_frames: 0,
			total_ticks: 0,
			state: LoopState::Pre,
		}
	}
	
	/// Changes the tick-rate to the given number.
	pub fn set_ticks_per_second(&mut self, ticks_per_second: i32) {
		self.ticks_per_second = ticks_per_second;
		self.skip_ticks = 1.0 / (ticks_per_second as f64);
	}
	
	pub fn update<TIME>(&mut self, time: TIME) -> LoopState
		where TIME: Fn() -> f64
	{
		self.state = match self.state {
			LoopState::Pre => {
				self.loops = 0;
				LoopState::TickCheck
			},
			
			LoopState::TickCheck => {
				let time = time();
				if (time > self.next_game_tick) && (self.loops < self.max_frameskip) {
					LoopState::Tick(self.ticks_per_second, time, self.skip_ticks as f32)
				} else {
					LoopState::FrameCheck
				}
			},
			
			LoopState::Tick(_, _, _) => {
				self.next_game_tick += self.skip_ticks;
				self.loops += 1;
				self.tick_count += 1;
				self.total_ticks += 1;
				LoopState::TickCheck
			},
			
			LoopState::FrameCheck => {
				let now = time();
				let delta = now - self.next_game_tick;
				
				let interpolation = (delta + self.skip_ticks) / self.skip_ticks;
				LoopState::Frame(now, interpolation)
			}
			
			LoopState::Frame(frame_start, _) => {
				let frame_end = time();
				self.frame_time = frame_end - frame_start;
				self.frame_count += 1;
				self.total_frames += 1;
				
				if frame_end - self.last_chk >= 1.0 && self.frame_count > 10 {
					// calculate averages
					let last_fps = self.frame_count as f64 / (frame_end - self.last_chk);
					let last_tps = self.tick_count as f64 / (frame_end - self.last_chk);
					
					// reset timer
					self.frame_count = 0;
					self.tick_count = 0;
					self.last_chk = frame_end;
					
					LoopState::Timer(
						last_fps,
						last_tps
					)
				} else {
					LoopState::Pre
				}
			},
			
			LoopState::Timer(_, _) => {
				LoopState::Pre
			},
			
			LoopState::Stop => {
				LoopState::Stop
			}
		};
		
		// Return the state
		self.state
	}
	
	pub fn get_ticks_per_second(&self) -> i32 {
		self.ticks_per_second
	}
	
	/// Returns the total number of ticks since the creation of the gameloop instance.
	pub fn get_total_ticks(&self) -> u64 {
		self.total_ticks
	}
	
	/// Returns the total number of frames since the creation of the gameloop instance.
	pub fn get_total_frames(&self) -> u64 {
		self.total_frames
	}
	
	/// Gets the time (in seconds) of the previous frame.
	pub fn get_last_frame_time(&self) -> f64 {
		self.frame_time
	}
	
	pub fn stop(&mut self) {
		self.state = LoopState::Stop
	}
}

/// Represents the loop-phase state of a gameloop.
#[derive(Clone, Copy)]
pub enum LoopState {
	/// The beginning of a frame.
	Pre,
	
	/// Decides whether more ticks should be done or to jump to `FrameCheck`.
	TickCheck,
	
	/// Tells the client to compute a new world-state.
	Tick(i32, f64, f32),
	
	/// Prepares the information for the Frame-state.
	FrameCheck,
	
	/// Tells the client to render a new frame and swap.
	Frame(f64, f64),
	
	/// Timing information; returned every once a second.
	Timer(f64, f64),
	
	// There is no state for the end of a frame.
	
	/// Represents a request to stop the gameloop.
	Stop,
}

// These are an important part of the public interface.
pub use LoopState::Tick;
pub use LoopState::Frame;
pub use LoopState::Timer;
pub use LoopState::Stop;
