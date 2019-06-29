# Talecraft Game Engine

The **Talecraft Game Engine** for discrete voxels (blocks!).

> Notice: **This project is not associated with *Mojang AB* in any way**.  
> The name 'talecraft' will be changed once a better name is found.

A discrete voxel is also commonly called *Block*, *Bloxel* or *Cubic Voxel*.

## Project Goals

> Nothing is truly set in stone except these goals.

Make it as data-driven as possible...

* No hardcoded ID's, wherever possible.
* No hardcoded gameplay elements.
* Make use of emergent systems.
* Make heavy use of scripting.

The final goal \(in the far future\) is to create the game called Talecraft.

## Architecture

The engine is split into several parts:

- The binary crate `tcge-client` contains all code for the game-client.
- The binary crate `tcge-server` contains all code for the game-server.
- The library crates contain, respectively...
  - `tcge-common`: Common utilities shared by all crates.
    - Everything too small for it's own crate goes here.
  - `tcge-backbone`: Component-based hierarchical state-machine.
  - `tcge-blocks`: Everything directly related to blocks.
  - `tcge-opengl`: The OpenGL bindings for the client.

### Backbone

> The source-code for the backbone can be found in the `tcge-backbone` crate.

The client makes use of the `tcge-backbone` crate to abstract
its structure, so that (global) state-keeping is reduced and concentrated
into the various components attached to the backbone.

An interesting side-effect of this, is that application-state can be
'linked to', by writing out the path in the backbone tree as string
and applying generic 'filesystem' path semantics to it.

## Build Instructions

1. Install the [rust-lang toolchain](https://rustup.rs/) for your OS.
2. Ensure the [prerequisites](https://crates.io/crates/glfw#prerequisites) for the GLFW crate.
	* For windows, you can use the precompiled binaries  
		intended for the MS Build-Tools (VS-2015 or higher).
3. Clone (or fork) the project into an empty directory.
4. In the project directory, run the command `cargo build`.
5. Done (?).

## Default Controls

| Button | Action |
|--------|--------|
| Press `ESC` twice. | Close game. |
| `M`       | (Un)Grab the mouse.  |
| `W/A/S/D` | Move forward/left/backward/right. |
| `L-SHIFT` | Move faster. |
| `L-CTRL`  | Move down.   |
| `SPACE`   | Move up.     |
| `C`                  | Change camera movement mode. |
| `Mouse Movement`     | Look around/aim. |
| `Left Mouse Button`  | Destroy blocks. |
| `Right Mouse Button` | Place blocks. |

## Console Commands

The game can read and process commands typed into the standard console.

| Command | Action |
|---------|--------|
| `stop` | Immediately quits the game. Same as pressing ESC twice. |
| `set-tps <number>` | Change the rate of ticks-per-second to an arbitrary value. |
| `loc <path>` | Change the backbone's current path. |

## Contributions

Generally welcome! (Always keep the project goals in mind!)

## License

MIT License! See [LICENSE](LICENSE).
