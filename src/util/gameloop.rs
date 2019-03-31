/// Represents a gameloop; contains the necessary state.
pub struct GameloopState {
	/// The normal duration between individual ticks.
	skip_ticks: f64,
	
	/// How many frames can be skipped when catching up with lost ticks.
	max_frameskip: i32,
	
	/// The moment when the next tick happens.
	next_game_tick: f64,
	
	/// Counter of catch-up ticks per frame.
	loops: i32,
	
	/// The interpolation factor of the last frame.
	interpolation: f64,
	
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
	
	/// The last measured average frames-per-second.
	last_fps: f64,
	
	/// The last measured average ticks-per-second.
	last_tps: f64,
	
	/// Whether to log timing information once a second.
	print_timers: bool,
}

impl GameloopState {
	/// Creates a new gameloop.
	///
	/// # Parameters
	///
	/// `ticks_per_second`: The amount of 'ticks' per second the gameloop should attempt to run.
	///
	/// `print_timers`: If true, the gameloop will print timing information to the console/terminal every once a second.
	pub fn new(ticks_per_second: i32, print_timers: bool) -> GameloopState {
		GameloopState {
			skip_ticks: 1.0 / (ticks_per_second as f64),
			max_frameskip: 5,
			loops: 0,
			tick_count: 0,
			next_game_tick: 0.0,
			interpolation: 0.0,
			frame_time: 0.0,
			frame_count: 0,
			last_chk: 0.0,
			last_fps: 0.0,
			last_tps: 0.0,
			total_frames: 0,
			total_ticks: 0,
			print_timers,
		}
	}
	
	// TODO: Make this function a iterator.
	// ALT: Make this function a generator (once possible).
	
	/// Applies any number of ticks trough the tick-callback and then calls the draw-callback once.
	/// This function requires a 'timer'-closure yielding high-resolution time in seconds.
	///
	/// # Parameters
	///
	/// `gtc`: A closure that returns time as seconds in a high-resolution.
	///
	/// `game_tick`: The closure to call when a tick occurs;
	/// receives the current time in seconds.
	///
	/// `game_draw`: The closure to call when a frame occurs;
	/// receives the current time in seconds and the current interpolation factor.
	/// This closure is called *once* for *every* call to this function.
	pub fn next<GTC, GT, GD> (
		&mut self,
		gtc: GTC,
		mut game_tick: GT,
		mut game_draw: GD
	) where
		GTC: Fn() -> f64,
		GT: FnMut(f64),
		GD: FnMut(f64, f32),
	{
		let frame_start = gtc();
		self.loops = 0;
		
		while (gtc() > self.next_game_tick) && (self.loops < self.max_frameskip) {
			game_tick(gtc());
			
			self.next_game_tick += self.skip_ticks;
			self.loops += 1;
			self.tick_count += 1;
			self.total_ticks += 1;
		}
		
		let now = gtc();
		let delta = now - self.next_game_tick;
		
		self.interpolation = (delta + self.skip_ticks) / self.skip_ticks;
		game_draw(now, self.interpolation as f32);
		
		let frame_end = gtc();
		self.frame_time = frame_end - frame_start;
		self.frame_count += 1;
		self.total_frames += 1;
		
		if frame_end - self.last_chk > 1.0 && self.frame_count > 10 {
			// calculate averages
			self.last_fps = self.frame_count as f64 / (frame_end - self.last_chk);
			self.last_tps = self.tick_count as f64 / (frame_end - self.last_chk);
			
			// reset timer
			self.frame_count = 0;
			self.tick_count = 0;
			self.last_chk = frame_end;
			
			if self.print_timers {
				debug!("{} FPS, {} TPS", self.last_fps, self.last_tps);
			}
		}
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
	pub fn get_frame_time(&self) -> f64 {
		self.frame_time
	}
	
	/// Gets the average rate of frames-per-second.
	pub fn get_frames_per_second(&self) -> f64 {
		self.last_fps
	}
	
	/// Gets the average rate of ticks-per-second.
	pub fn get_ticks_per_second(&self) -> f64 {
		self.last_tps
	}
}