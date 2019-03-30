use super::node;
use super::comp;
use super::lens;
use std::any::TypeId;

pub struct Context<'a> {
	pub lens: &'a lens::Lens,
	pub nodes: &'a mut node::Nodes,
}

impl<'a> Context<'a> {
	pub fn new<'b>(lens: &'b lens::Lens, nodes: &'b mut node::Nodes) -> Context<'b> {
		Context {lens, nodes}
	}
	
	pub fn get_lens_name(&self) -> &str {
		self.lens.name.as_str()
	}
	
	pub fn get_lens_state(&self) -> &lens::State {
		&self.lens.state
	}
	
	pub fn get_lensed_node_id(&self) -> Option<usize> {
		match self.lens.path.last() {
			Some(node_id) => Some(*node_id),
			None => None
		}
	}
	
	pub fn get_lensed_node(&self) -> Option<&node::Node> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_node_by_id(node_id),
			None => None
		}
	}
	
	pub fn get_lensed_node_name(&self) -> Option<&str> {
		match self.get_lensed_node_id() {
			Some(node_id) => match self.nodes.get_node_by_id(node_id) {
				Some(node) => Some(node.name.as_str()),
				None => None
			},
			None => None
		}
	}
	
	pub fn get_path(&self) -> &[usize] {
		self.lens.path.as_slice()
	}
	
	pub fn get_path_str(&self) -> &str {
		self.lens.path_str.as_str()
	}
	
	pub fn get_component(&self, component_type_id: TypeId) -> Result<&'a comp::Component, comp::ComponentAccessError> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_node_component(node_id, component_type_id),
			None => Err(comp::ComponentAccessError::NodeNotFound{node_id:0})
		}
	}
	
	pub fn get_mut_component(&mut self, component_type_id: TypeId) -> Result<&'a mut comp::Component, comp::ComponentAccessError> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_mut_node_component(node_id, component_type_id),
			None => Err(comp::ComponentAccessError::NodeNotFound{node_id:0})
		}
	}
	
	pub fn get_component_downcast<C: comp::Component>(&self) -> Result<&'a C, comp::ComponentAccessError> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_node_component_downcast::<C>(node_id),
			None => Err(comp::ComponentAccessError::NodeNotFound{node_id:0})
		}
	}
	
	pub fn get_mut_component_downcast<C: comp::Component>(&mut self) -> Result<&'a mut C, comp::ComponentAccessError> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_mut_node_component_downcast::<C>(node_id),
			None => Err(comp::ComponentAccessError::NodeNotFound{node_id:0})
		}
	}
	
	pub fn fire_event(&mut self) {
		// TODO: Implement event handling for the context.
		// Possibly by splitting it out from the router itself?
		panic!("Not yet implemented.");
	}
}