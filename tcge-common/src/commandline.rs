use std::sync::mpsc;
use std::thread;
use std::io::stdin;

type Message = String;

pub struct CommandLine {
	pub handle: thread::JoinHandle<()>,
	pub pipe: mpsc::Receiver<Message>,
}

impl CommandLine {
	pub fn new() -> Self {
		let (pipe, recv) = mpsc::channel();
		
		let thread = thread::Builder::new()
			.name("CMD-Reader".to_string())
			.spawn(move || {
				let pipe = pipe;
				work(pipe);
			})
			.unwrap();
		
		Self {
			handle: thread,
			pipe: recv
		}
	}
	
	pub fn recv(&self) -> Option<Message> {
		let recv = self.pipe.try_recv();
		
		let recv = match recv {
			Err(err) => {
				match err {
					mpsc::TryRecvError::Empty => {
						return None
					},
					mpsc::TryRecvError::Disconnected => {
						return None
					},
				}
			},
			Ok(msg) => msg
		};
		
		Some(recv)
	}
}

pub fn work(pipe: mpsc::Sender<Message>) {
	let stdin = stdin();
	let mut buffer = String::new();
	
	while let Ok(_) = stdin.read_line(&mut buffer) {
		let command = buffer.trim().to_string();
		
		if let Err(e) = pipe.send(command) {
			error!("Unable to send command to main-thread: {}", e);
			break;
		}
		
		buffer.clear();
	}
}
