use super::rustc_hash::FxHashMap;
use super::lens::MoveEvent;
use std::any::TypeId;

pub struct Node {
	pub parent: Option<usize>,
	pub name: String,
	pub id: usize,
	lens_count:usize,
}

impl Node {
	pub fn new(id: usize, parent: Option<usize>, name: &str) -> Node {
		let name = name.to_string();
		Node {
			id,
			parent,
			name,
			lens_count: 0,
		}
	}
	
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
	
	pub fn is_child_of(&self, parent: &Node) -> bool {
		self.parent.map_or(false, |id| id == parent.id)
	}
	
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

pub struct Nodes {
	pub nodes: FxHashMap<usize, Node>, // TODO: This is really just a `Map<usize, Node>` ...
	pub next_id: usize,
}

impl Nodes {
	
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
			next_id: 1
		}
	}
	
	pub fn next_id(&mut self) -> usize {
		let id = self.next_id;
		self.next_id += 1;
		return id;
	}
	
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
	
	pub fn get_mut_node_by_id(&mut self, id: usize) -> Option<&mut Node> {
		self.nodes.get_mut(&id)
	}
	
	pub fn get_node_by_id(&self, id: usize) -> Option<&Node> {
		self.nodes.get(&id)
	}
	
	pub fn get_node_id(&self, name: &str) -> Option<usize> {
		for (pos, node) in self.nodes.iter() {
			if node.name == name {
				return Some(*pos)
			}
		}
		
		None
	}
	
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