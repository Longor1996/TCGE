# Talecraft Game Engine

> A game engine for playing with block-shaped voxels.

This project is currently at the beginning of it's development.

## Project Goals

> Nothing is truly set in stone except these goals.

Make it as data-driven as possible...

* No hardcoded ID's, wherever possible.
* No hardcoded gameplay elements.
* Make use of emergent systems.
* Make heavy use of CDML.

The final goal \(in the far future\) is to create the game called Talecraft.

## Project Structure

The program consists of three parts: two 'binary' modules and several 'library' modules.

The binary modules directly depend on their respective library-module:

- `src/bin/client.rs`: The entry-point and mainloop for the client binary.
- `src/bin/server.rs`: The entry-point and mainloop for the server binary.
- `src/client/`: The library containing the modules for the client.
- `src/server/`: The library containing the modules for the server.

All other library modules essentially represent the various systems of the engine,
with the sole exception of the `router`-module which is the frameworks backbone.

## Build Instructions

1. Install the [rust-lang toolchain](https://rustup.rs/) for your OS.
2. Ensure the [prerequisites](https://crates.io/crates/glfw#prerequisites) for the GLFW crate.
	* For windows, you can use the precompiled binaries  
		intended for the MS Build-Tools (VS-2015 or higher).
3. Clone (or fork) the project into an empty directory.
4. In the project directory, run the command `cargo build`.
5. Done (?).
