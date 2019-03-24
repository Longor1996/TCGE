use super::rustc_hash::FxHashMap;

pub struct Node {
	pub parent: Option<usize>,
	pub name: String,
	pub id: usize,
}

impl Node {
	pub fn on_event(&mut self, _event: &mut super::event::Wrapper) {
		// TODO: Walk trough event-listeners/components...
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