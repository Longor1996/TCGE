//! Module for managing the various commandline arguments/options of the client.

extern crate clap;

use self::clap::{Arg, App};

/// Holds the parsed and ready-to-use commandline-options for the client.
pub struct CmdOptions {
	pub path: String,
	pub width: u32,
	pub height: u32,
	pub gl_debug: bool,
	pub gl_multisamples: u32,
}

pub fn parse() -> Result<CmdOptions, failure::Error> {
	let matches = App::new("tcge-client")
		.version(env!("VERSION"))
		.author("Lars Longor K <lalongok@gmail.com>")
		.about("A game-engine for discrete voxels.")
		// beginning of command line configuration
		// TODO: Add commandline options for various things...
		
		.arg(Arg::with_name("width")
			.help("Sets the width of the primary window.")
			.long("width")
			.value_name("WIDTH")
			.takes_value(true)
			.require_equals(true)
			.default_value("1024")
			.validator(|v: String| {
				v.parse::<u32>()
					.map(|_val| ())
					.map_err(|err| err.to_string())
			})
		)
		
		.arg(Arg::with_name("height")
			.help("Sets the height of the primary window.")
			.long("height")
			.value_name("HEIGHT")
			.takes_value(true)
			.require_equals(true)
			.default_value("768")
			.validator(|v: String| {
				v.parse::<u32>()
					.map(|_val| ())
					.map_err(|err| err.to_string())
			})
		)
		
		.arg(Arg::with_name("gl_multisample")
			.help("Sets the amount of samples to use for the framebuffer.")
			.long("gl_multisample")
			.value_name("GL_SAMPLES")
			.takes_value(true)
			.require_equals(true)
			.validator(|v: String| {
				v.parse::<u32>()
					.map(|_val| ())
					.map_err(|err| err.to_string())
			})
		)
		
		.arg(Arg::with_name("gl_debug")
			.long("gl_debug")
			.help("Enables OpenGL debugging.")
		)
		
		.arg(Arg::with_name("PATH")
			.help("Where to navigate to when the client-lens is created.")
			.index(1)
			.default_value("/")
		)
		
		// end of command line configuration
		.get_matches();
	
	Ok(CmdOptions {
		path: matches.value_of("PATH").unwrap_or("/").to_string(),
		
		width: matches.value_of("WIDTH")
			.unwrap_or("1024").parse::<u32>()?
		,
		height: matches.value_of("HEIGHT")
			.unwrap_or("768").parse::<u32>()?
		,
		gl_debug: matches.is_present("gl_debug")
		,
		gl_multisamples: matches.value_of("GL_SAMPLES")
			.unwrap_or("0").parse::<u32>()?
		,
	})
}
