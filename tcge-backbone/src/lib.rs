//! The backbone represents, owns and manages the architecture and internal structure of the entire engine.
//!
//! It handles all application state, data and code by splitting it into components attached to nodes with handlers.
//!
//! This is done by representing the application as a tree of 'nodes',
//! over which the 'client' can move and interact with the app trough a 'context'.

extern crate rustc_hash;
use rustc_hash::FxHashMap;

#[macro_use]
extern crate mopa;

mod nodes;
mod comps;
mod state;
mod event;

pub use nodes::*;
pub use comps::*;
pub use state::*;
pub use event::*;
use core::borrow::BorrowMut;

/// The path of a backbone.
pub type Path = Vec<NodeId>;

/// Creates a new backbone instance.
pub fn new() -> Backbone {
	Backbone::new()
}

/// The actual backbone instance that binds everything together.
pub struct Backbone {
	
	/// The NodeId of the root node.
	root_id: NodeId,
	
	/// Counter for new NodeId instances.
	counter: usize,
	
	/// Collection of nodes.
	nodes: Nodes,
	
	/// Collection of collections of components.
	comps: Comps,
	
	/// Node event handlers.
	handlers: Handlers,
	
	path: Path,
	
	path_str: String,
	
	state: State,
}

// Functions for constructing the backbone
impl Backbone {
	pub fn new() -> Self {
		let root_id = nodes::NodeId::new(0);
		let root = Node::new(root_id, "", root_id);
		
		let mut nodes = FxHashMap::default();
		let comps = FxHashMap::default();
		
		let handlers = FxHashMap::default();
		
		nodes.insert(root_id, root);
		
		Self {
			root_id,
			counter: 0,
			nodes,
			comps,
			handlers,
			path: vec![],
			path_str: String::new(),
			state: State::Idle,
		}
	}
}

// Functions for location & state management
impl Backbone {
	
	pub fn location_set(&mut self, path: &str) -> Result<(), ()> {
		if self.state.can_replace() {
			self.state = State::Move(path.to_string(), 0);
			Ok(())
		} else {
			Err(())
		}
	}
	
	pub fn location_get(&self) -> &Vec<NodeId> {
		&self.path
	}
	
	pub fn location_get_node(&self) -> Option<NodeId> {
		match self.path.last() {
			Some(x) => Some(*x),
			None => None
		}
	}
	
	pub fn location_get_str(&self) -> &str {
		self.path_str.as_str()
	}
	
	pub fn update_until_idle(&mut self) {
		loop {
			self.update();
			if self.state == State::Idle {
				return
			}
		}
	}
	
	pub fn update(&mut self) -> bool {
		// Stop if necessary
		if let State::Stop(_) = self.state {
			return false
		}
		
		if self.path.is_empty() && self.state.can_replace() {
			self.state = State::Move("/".to_string(), 0);
		}
		
		if let State::Move(path, offset) = self.state.borrow_mut() {
			
			let step = update_path(
				&self.nodes,
				path.as_str(),
				offset,
				&self.path
			);
			
			let old_path_len = self.path.len();
			
			let new_state: Option<State> = match step {
				PathChange::ToRoot => {
					self.path.clear();
					self.path.push(self.root_id);
					None
				},
				
				PathChange::ToSelf => {
					None
				},
				
				PathChange::ToSuper => {
					self.path.pop();
					None
				},
				
				PathChange::ToNode(id) => {
					self.path.push(id);
					None
				},
				
				PathChange::Error(reason) => {
					Some(State::Stop(Some(format!("Failed to change path: {}", reason))))
				},
				
				PathChange::End => {
					Some(State::Idle)
				},
			};
			
			let new_path_len = self.path.len();
			
			if old_path_len != new_path_len {
				self.path_str = self.path_to_string(&self.path).expect("Failed to resolve path");
			}
			
			if let Some(state) = new_state {
				self.state = state
			}
		}
		
		true
	}
	
	pub fn get_state(&self) -> &State {
		&self.state
	}
	
	pub fn stop(&mut self) {
		self.state = State::Stop(None)
	}
}
