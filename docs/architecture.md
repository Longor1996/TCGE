This document will contain an overview of the engine architecture.

## Overview

The engine consists of a few interacting systems.

### The Backbone
The Backbone is a hierarchy of named nodes, over which *monitors/lenses* move asynchronously.

Interfacing with the backbone is both instant (due to asynchronity) and threadsafe at all times.

The contents of the backbone are mostly defined by the engine,
though additional nodes may be attached trough the brigadier.

The nodes of the backbone by themself cannot do anything.

Whenever a monitor moves to another node, the node may load arbitrary resources and modules.
On the other hand, when a monitor leaves a node, the node may unload the things it loaded.

Since this process is entirely asynchronous, the active monitors can keep on doing whatever,
until their request to access a node has completed, upon which they may act.

What exactly is loaded by a node depends:
- On the type of the monitor and any of its sub-monitors.
- The things attached to the node by [#The Brigadier].

### The Plumber
The Plumber is a message-passing system that *plumbs* messages between systems & modules.
It does all of its work on top of [#The Backbone].
It's behaviour is defined by [#The Brigadier].



### The Brigadier
The Brigadier is a reactive/functional system for *defining* stuff, *any* stuff, of all forms and shapes.

When the engine first starts up, it will load two files:
- `sys_init.cdml`: Stored inside the engines executable, this file defines the core content.
- `usr_init.cdml`: Stored beside the engines executable, this file is entirely user-defined.

Both files are `Content Library` files written in the `CDML` format.
Once loaded, the brigadier will first compile and validate, then finally 'execute' the files.

> **Note:**
> The engine may cache the compiled result of the content libraries for faster startup.
> It will, however, always validate the content libraries for correctness before execution.

A content-library is not executed in the sense that the brigadier will walk trough instructions step-by-step,
but rather by the brigadier *reacting* to events and activating the relevant definitions inside the libraries.

### The Provider
The Provider is a read-only, package-based, layered, virtual filesystem tied to the backbone.

Whenever any system needs a resource, a request in the form of a `ResourceLocation`
is given to the provider, who goes trough all layers and returns the first matching asset.

Access to the provider is possible at all times trough the backbone.

A layer is a virtual filesystem, 'layered' on top of some real filesystem.
There are multiple types of virtual filesystems:
- `Engine`: The engine executable itself holds hardcoded resources.
- `ConLib`: The raw data section of some arbitrary `CDML` file.
- `Folder`: A folder that contains assets and an index file.
- `Archive`: A archive (eg: a ZIP) that contains assets and an index file.

With the exception of the `Engine`-filesystem, all the others are linked in by the brigadier.

A `ResourceLocation` is defined as a pair of an optional package-name and a path.

If a `ResourceLocation` is created without a package-name, the `core`-package is selected.

The API of the provider looks like the following:

```
fn Provider.getAsset(
	location: ResourceLocation,
	superdir: String,
	formats: Vec<String>
) -> Result<AssetReference,ProviderError>

fn AssetReference.asBuffer() -> Vec<u8>
fn AssetReference.asStream() -> Reader
```
























---

# Implement Backbone Architecture

> This document is a work in progress, as the architecture is still being 'invented'.

Progress:
- [x] Basic structure and event handling (e4b98f271bc5f2f026957d750b9e814c6fc79ebc)
- [x] Receiving of events and shutdown procedure (2d53353bcff5694f3a6c7a90f010f7f74f9d2008)
  - First successfull test of the routing backbone.
- [x] Resolving of paths, with optional bubbling (202762c32bef8904fcc39cac3125f813d8479c2b).
- [x] Lens movement trough the routing tree (58a73f729002e36251fdf6bf9472b7c7a178546c)
- [x] Implement node components.
- [x] Let lens-handlers fetch components.

# 'Backbone' Architecture

The backbone represents and manages the architecture and internal structure of the entire engine. Instantiation of all systems and modules (except the game's mainloop) happens trough it.

It is constructed in the form of a tree of named nodes, which can be queried and monitored trough paths or accessed directly by their unique ID. The user-interface (be it a client window or server console) creates a `Lens` that is is a pointer into the tree; the movement of this pointer causes node-components to either load or unload.

At the moment the engine starts, the users lens 'walks' from nothing (` `) to the root (`/`) of the tree, causing the loading process to begin, constructing the actual user-interface and eventually showing the main-menu. The entire UI thus consists of (potentially dynamic) links between the various nodes.

## Nodes

Nodes are what the backbone is made of. They are simple generic objects that can modify themselves arbitrarely.

A node has the following basic structure:
```rust
struct Node {
  id: usize,
  name: String,
  parent: Optional<usize>,
}
```

A node can define itself as 'fallback' for its sibling nodes. The tree-walker collects all fallback nodes, and if a given named node cant be found in the current node, it will look trough the fallback list as last resort.

## Paths

The actual pointer of a monitor is a path, just a string, with a specific syntax that can be changed trough user-interaction and various events. Changing the path in *any* way causes the monitor to re-evaluate where it is in the tree, making the backbone load/unload things.

The syntax for paths can be expressed as the following EBNF:
```ebnf
path: entry || item ('/' item)*
  entry: '/' | '~/' | './' | '../'
  item: '#'? (word | '*') args?
  args: '?' (arg ('&' arg)*)?
  arg: word ('=' [a-z A-Z 0-9 + - . ,])
  word: [a-z A-Z][a-z A-Z 0-9 -]*
```

Paths can also be used for sending events to other nodes.

## Events

But it doesn't just stop there. All nodes that are part of the active tree (eg: all nodes under the path and their siblings) can send events to other nodes in the tree, again causing things to be loaded/unloaded. Events are temporary monitors that, during their short lifetime, go trough a set of phases. The phases of an event are thusly defined:

- `CAPTURE`-Phase: The event has been fired from a node, and walks trough the tree, node by node, until it reaches it's target. If it can't reach it's target, due to the event being caught or cancelled, it *can* immediately go into the `BUBBLE` phase.
- `ACTION`-Phase: The event has reached it's target and is being processed by the event handlers defined on the node. After the action has been completed, the event will optionally enter the `BUBBLE` phase.
- `BUBBLE`-Phase. The event walks back, trough the tree, to the node that fired it, evaluating any event-handlers flagged for the bubbling-phase.

Sending an event to another node is a fire-and-forget operation; once the function returns the event is owned by the backbone and can no longer be accessed.

## Lenses

A lens is a stringly-named pointer into the routing tree, capable of moving around within it and also receiving and acting upon events fired into the routing tree.

## Why?

What's it good for? For globally managing the state of the engine in a consistent, concurrent and safe, but still dynamic manner. It is a 'one time' investment similar to URL-rewriting in a web-server.

Due to the backbone *owning* the application/process/engine-structure, managing memory and instance lifetimes also becomes easier to deal with.

## Planned Structure

If all of the above seems weird, maybe the planned backbone structure will help with understanding the system introduced by this issue.

```
.   (loading screen)
./  (main menu)

./user
./user/#login
./user/#register
./user/<USER-ID>

./localplayer
./localplayer/#create
./localplayer/#create/gameplay
./localplayer/#create/terrain
./localplayer/#create/addons
./localplayer/#create/npcs
./localplayer/#createFromUrl
./localplayer/#createFromBackup
./localplayer/#createFromArchive

./localplayer/<WORLD-DIR>
./localplayer/<WORLD-DIR>/#generating
./localplayer/<WORLD-DIR>/#recreateWorld
./localplayer/<WORLD-DIR>/#backupProcess
./localplayer/<WORLD-DIR>/#confirmDelete
./localplayer/<WORLD-DIR>/#options/gameplay
./localplayer/<WORLD-DIR>/#options/npcs
./localplayer/<WORLD-DIR>/play [REF: */play]
./localplayer/<WORLD-DIR>/play/#openToLan [FALLING]

./multiplayer
./multiplayer/#add
./multiplayer/#join

./multiplayer/<HOST-ADDR>
./multiplayer/<HOST-ADDR>/#confirmDelete
./multiplayer/<HOST-ADDR>/manage
./multiplayer/<HOST-ADDR>/chat
./multiplayer/<HOST-ADDR>/play [REF: */play]

*/play/warp
*/play/<DIM>
*/play/*****/#menu
*/play/*****/#inventory
*/play/*****/#advancements
*/play/*****/#recipes
*/play/*****/#<BLOCKPOS>/
*/play/*****/#<ACTOR-ID>/

./community/
./community/<ITEM>/
./community/search/
./community/#downloads [FALLING]
./community/#uploads   [FALLING]
./community/#uploads/new

./#console [FALLING]

./#options [FALLING]
./#options/audio
./#options/avatar
./#options/network
./#options/graphics
./#options/controls
./#options/reporter
./#options/interface
./#options/languages
./#options/overrides
./#options/overrides/#addFromUrl
./#options/overrides/#addFromArchive
```