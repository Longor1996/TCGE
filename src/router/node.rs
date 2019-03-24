use super::rustc_hash::FxHashMap;
use super::lens::MoveEvent;
use super::comp;
use std::any::TypeId;

/// Represents a single named unique node in the routing-tree.
pub struct Node {
	pub parent: Option<usize>,
	pub name: String,
	pub id: usize,
	lens_count:usize,
}

impl Node {
	/// Creates a new node to be inserted into the routing-tree.
	pub fn new(id: usize, parent: Option<usize>, name: &str) -> Node {
		let name = name.to_string();
		Node {
			id,
			parent,
			name,
			lens_count: 0,
		}
	}
	
	/// Function for processing and acting upon received events.
	pub fn on_event(&mut self, components: Option<&mut FxHashMap<TypeId, Box<super::comp::Component>>>, event: &mut super::event::Wrapper) {
		event.event.downcast_ref::<MoveEvent>().map(| move_event | {
			let old_lens_count = self.lens_count;
			
			match move_event {
				MoveEvent::EnterNode => {self.lens_count += 1},
				MoveEvent::LeaveNode => {self.lens_count -= 1},
			};
			
			if old_lens_count != self.lens_count {
				if self.lens_count == 0 {
					if let Some(components) = components {
						for (_, component) in components {
							(*component).on_unload();
						}
					}
				} else {
					if let Some(components) = components {
						for (_, component) in components {
							(*component).on_load();
						}
					}
				}
			}
		});
		
		// Ignore all other events, for now.
	}
	
	/// Test if this node is a child of the given node.
	pub fn is_child_of(&self, parent: &Node) -> bool {
		self.parent.map_or(false, |id| id == parent.id)
	}
	
	/// Test if this node has the given specific name.
	pub fn is_named(&self, name: &str) -> bool {
		self.name == name
	}
}

impl PartialEq for Node {
	/// Partial equality for nodes, using their ID's.
	fn eq(&self, other: &Node) -> bool {
		self.id == other.id
	}
}

/// Container for all nodes and components.
pub struct Nodes {
	/// Owner of all nodes and thus the routing-tree.
	pub nodes: FxHashMap<usize, Node>,
	
	/// Container for node components.
	pub comps: comp::Components,
	
	/// Auto-incrementing counter for the next unique node-id.
	pub next_id: usize,
}

impl Nodes {
	/// Effectively creates a new routing tree, though without routing and lenses.
	pub fn new() -> Nodes {
		let root_node = Node {
			id: 0,
			parent: None,
			name: "".to_string(),
			lens_count: 0,
		};
		
		let mut nodes = FxHashMap::default();
		nodes.insert(root_node.id, root_node);
		
		Nodes {
			nodes,
			next_id: 1,
			comps: comp::Components::new(),
		}
	}
	
	/// Returns the next unique node-id.
	pub fn next_id(&mut self) -> usize {
		let id = self.next_id;
		self.next_id += 1;
		return id;
	}
	
	/// Returns a formatted string representing the given path.
	pub fn get_path_as_string(&self, path: &[usize]) -> Result<String, ()> {
		let mut path_str = String::new();
		path_str += "/";
		
		for item in path {
			let node = self.get_node_by_id(*item);
			if let Some(node) = node {
				path_str += node.name.as_str();
			} else {
				return Err(());
			}
		}
		
		if path_str.len() > 1 {
			path_str.trim_end_matches("/");
		}
		
		Ok(path_str)
	}
	
	/// Mutably borrow the node with the given id.
	pub fn get_mut_node_by_id(&mut self, id: usize) -> Option<&mut Node> {
		self.nodes.get_mut(&id)
	}
	
	/// Mutably borrow the node (and its components) with the given id.
	pub fn get_mut_node_with_comps_by_id(&mut self, id: usize) -> (Option<&mut Node>, Option<&mut FxHashMap<TypeId, Box<comp::Component>>>) {
		(self.nodes.get_mut(&id), self.comps.comps.get_mut(&id))
	}
	
	/// Borrow the node with the given id.
	pub fn get_node_by_id(&self, id: usize) -> Option<&Node> {
		self.nodes.get(&id)
	}
	
	/// Get the id of the parent of the given node.
	pub fn get_node_parent_id(&self, node_id: usize) -> Option<usize> {
		match self.nodes.get(&node_id) {
			Some(node) => {
				match node.parent {
					Some(parent) => Some(parent),
					None => None
				}
			},
			None => None
		}
	}
}