use super::rustc_hash::FxHashMap;
use core::borrow::{Borrow, BorrowMut};
use std::any::TypeId;

/// Container for all components bound to nodes.
pub struct Components {
	/// Collection of collections of components.
	pub comps: FxHashMap<usize, FxHashMap<TypeId, Box<Component>>>,
}

impl Components {
	/// Creates a new empty container for components.
	pub fn new() -> Components {
		Components {
			comps: FxHashMap::default()
		}
	}
}

impl super::node::Nodes {
	/// Set a component of a specific type for the given node.
	pub fn set_node_component(&mut self, node_id: usize, component: Box<Component>) -> bool {
		let component_type_id = mopa::Any::get_type_id(&component);
		
		// --- If the node has no components...
		if ! self.comps.comps.contains_key(&node_id) {
			let mut new_components = FxHashMap::default();
			new_components.insert(component_type_id, component);
			self.comps.comps.insert(node_id, new_components);
			
			// Let the component initialize things.
			( * self.comps.comps
				.get_mut(&node_id)
				.unwrap()
				.get_mut(&component_type_id)
				.unwrap()
			).on_attachment(node_id);
			return true;
		}
		
		let components = self.comps.comps.get_mut(&node_id).unwrap();
		if let Some(mut old) = components.insert(component_type_id, component) {
			old.on_detachment(node_id);
		}
		
		// Let the component initialize things.
		( * components
			.get_mut(&component_type_id)
			.unwrap()
		).on_attachment(node_id);
		
		return true;
	}
	
	/// Borrow a component of the given type from the given node, or any of its parents.
	pub fn get_node_component(&self, node_id: usize, component_type: TypeId) -> Option<&Component> {
		return match {self.comps.comps.contains_key(&node_id)} {
			false => match {self.get_node_parent_id(node_id)} {
				Some(next_id) => return self.get_node_component(next_id, component_type),
				None => None
			},
			true => {
				return self.comps.comps.get(&node_id).unwrap().get(&component_type).map(|boxed| {
					let borrow = boxed.borrow();
					borrow
				});
			}
		};
	}
	
	/// Mutably borrow a component of the given type from the given node, or any of its parents.
	pub fn get_mut_node_component(&mut self, node_id: usize, component_type: TypeId) -> Option<&mut Component> {
		return match {self.comps.comps.contains_key(&node_id)} {
			false => match {self.get_node_parent_id(node_id)} {
				Some(next_id) => return self.get_mut_node_component(next_id, component_type),
				None => None
			},
			true => {
				return self.comps.comps.get_mut(&node_id).unwrap().get_mut(&component_type).map(|boxed| {
					let borrow = boxed.borrow_mut();
					borrow
				});
			}
		};
	}
}





/// A component is a bundle of user-logic and -state attached to a node,
/// that can be loaded and unloaded depending on the residence of lenses.
pub trait Component: mopa::Any {
	/// Returns a engine internal (no i18n) name for the components type.
	fn get_type_name(&self) -> &'static str;
	
	/// This function is called when the component is attached to a node.
	fn on_attachment(&mut self, node_id: usize);
	
	/// This function is called when the component is detached from a node.
	/// Should not happen unless a component is replaced.
	fn on_detachment(&mut self, node_id: usize);
	
	fn on_load(&mut self);
	fn on_unload(&mut self);
}

// This is 100% necessary until `std::` provides Any for object-traits.
mopafy!(Component);

impl PartialEq for Component {
	fn eq(&self, other: &Component) -> bool {
		mopa::Any::get_type_id(self) == mopa::Any::get_type_id(other)
	}
}

impl Eq for Component {}

impl std::hash::Hash for Component {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		mopa::Any::get_type_id(self).hash(state);
	}
}
