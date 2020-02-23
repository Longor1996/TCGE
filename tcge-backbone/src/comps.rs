use std::any::TypeId;
use std::mem::transmute;
use core::borrow::{Borrow, BorrowMut};
use rustc_hash::FxHashMap;
use super::NodeId;
use super::Handler;

/// Collection type for `Component` instances, attached to `Node` instances by id.
pub type Comps = FxHashMap<NodeId, FxHashMap<TypeId, Box<dyn Component>>>;

// Implementation details regarding components.
impl<'a> super::Backbone {
	
	/// Attaches the given component to the given node (by id).
	///
	/// ## Unsafe Reference Warning
	///
	/// The returned reference is only valid for the current scope,
	/// and by replacing or deleting the component or the node it's attached to,
	/// the returned reference *will* become invalid.
	/// Only use it for further setup or as a scoped global.
	pub fn node_component_attach<C: Component>(&mut self, node_id: NodeId, component: C) -> &'a mut C {
		if ! self.nodes.contains_key(&node_id) {
			panic!("Could not attach component: The given NodeId is not valid.");
		}
		
		self.comps.entry(node_id).or_insert(FxHashMap::default());
		
		let type_id = component.get_type_id();
		let comps = self.comps.get_mut(&node_id).unwrap();
		
		if let Some(mut old) = comps.insert(type_id, Box::new(component)) {
			old.on_detachment(node_id);
		}
		
		comps.get_mut(&type_id).unwrap().on_attachment(node_id);
		
		match comps.get_mut(&type_id).unwrap().downcast_mut::<C>() {
			None => panic!("Could not fetch component immediately after attaching it."),
			Some(comp) => unsafe {
				// WARNING: Do not try this at home!
				transmute::<&mut C, &'a mut C>(comp.borrow_mut())
			}
		}
	}
	
	pub fn set_root_node_handler(&mut self, handler: Box<dyn Handler>) {
		self.handlers.insert(self.root_id, handler);
	}
}

impl<'a> super::Backbone {
	pub fn component_get<C: Component>(&self) -> Result<&'a C, ComponentAccessError> {
		let start_id = match self.location_get_node() {
			Some(x) => x,
			None => return Err(ComponentAccessError::PathIsNull)
		};
		
		get_component(&self.nodes, &self.comps, start_id)
	}
	
	pub fn component_get_by_node<C: Component>(&self, node_id: NodeId) -> Result<&'a C, ComponentAccessError> {
		get_component(&self.nodes, &self.comps, node_id)
	}
	
	pub fn component_get_mut<C: Component>(&mut self) -> Result<&'a mut C, ComponentAccessError> {
		let start_id = match self.location_get_node() {
			Some(x) => x,
			None => return Err(ComponentAccessError::PathIsNull)
		};
		
		get_component_mut(&self.nodes, &mut self.comps, start_id)
	}
	
	pub fn component_get_mut_by_node<C: Component>(&mut self, node_id: NodeId) -> Result<&'a mut C, ComponentAccessError> {
		get_component_mut(&self.nodes, &mut self.comps, node_id)
	}
}

impl<'a> super::Context<'a> {
	pub fn component_get<C: Component>(&self) -> Result<&'a C, ComponentAccessError> {
		get_component(&self.nodes, &self.comps, self.current)
	}
	
	pub fn component_get_by_node<C: Component>(&self, node_id: NodeId) -> Result<&'a C, ComponentAccessError> {
		get_component(&self.nodes, &self.comps, node_id)
	}
	
	pub fn component_get_mut<C: Component>(&mut self) -> Result<&'a mut C, ComponentAccessError> {
		get_component_mut(&self.nodes, &mut self.comps, self.current)
	}
	
	pub fn component_get_mut_by_node<C: Component>(&mut self, node_id: NodeId) -> Result<&'a mut C, ComponentAccessError> {
		get_component_mut(&self.nodes, &mut self.comps, node_id)
	}
}

fn get_component<'a, C: Component>(nodes: &super::Nodes, comps: &super::Comps, node_id: NodeId) -> Result<&'a C, ComponentAccessError> {
	match comps.get(&node_id) {
		None => {
			let node_parent_id = nodes.get(&node_id).unwrap().get_parent_id();
			if node_id == node_parent_id {
				Err(ComponentAccessError::DoesNotExist)
			} else {
				get_component(nodes, comps, node_parent_id)
			}
		},
		
		Some(node_comps) => {
			let type_id = TypeId::of::<C>();
			match node_comps.get(&type_id) {
				None => {
					let node_parent_id = nodes.get(&node_id).unwrap().get_parent_id();
					if node_id == node_parent_id {
						Err(ComponentAccessError::DoesNotExist)
					} else {
						get_component(nodes, comps, node_parent_id)
					}
				},
				
				Some(comp) => {
					match comp.downcast_ref::<C>() {
						None => Err(ComponentAccessError::CantDowncast),
						Some(comp) => unsafe {
							// WARNING: Do not try this at home!
							Ok(transmute::<&C, &'a C>(comp.borrow()))
						}
					}
				}
			}
		}
	}
}

fn get_component_mut<'a, C: Component>(nodes: &super::Nodes, comps: &mut super::Comps, node_id: NodeId) -> Result<&'a mut C, ComponentAccessError> {
	match comps.get_mut(&node_id) {
		None => {
			let node_parent_id = nodes.get(&node_id).unwrap().get_parent_id();
			if node_id == node_parent_id {
				Err(ComponentAccessError::DoesNotExist)
			} else {
				get_component_mut(nodes, comps, node_parent_id)
			}
		},
		
		Some(node_comps) => {
			let type_id = TypeId::of::<C>();
			match node_comps.get_mut(&type_id) {
				None => {
					let node_parent_id = nodes.get(&node_id).unwrap().get_parent_id();
					if node_id == node_parent_id {
						Err(ComponentAccessError::DoesNotExist)
					} else {
						get_component_mut(nodes, comps, node_parent_id)
					}
				},
				
				Some(comp) => {
					match comp.downcast_mut::<C>() {
						None => Err(ComponentAccessError::CantDowncast),
						Some(comp) => unsafe {
							// WARNING: Do not try this at home!
							Ok(transmute::<&mut C, &'a mut C>(comp.borrow_mut()))
						}
					}
				}
			}
		}
	}
}

/// Errors that may occur when accessing components.
pub enum ComponentAccessError {
	PathIsNull,
	DoesNotExist,
	CantDowncast,
}

impl std::fmt::Display for ComponentAccessError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "{}", match self {
			ComponentAccessError::PathIsNull => "Path is empty",
			ComponentAccessError::DoesNotExist => "Does not exist",
			ComponentAccessError::CantDowncast => "Cant downcast",
		})
	}
}

////////////////////////////////////////////////////////////////////////////////

/// A component is a bundle of user-logic and -state attached to a node.
pub trait Component: mopa::Any {
	/// Returns an engine internal (no i18n) name for the components type.
	fn get_type_name(&self) -> &'static str;
	
	/// This function is called when the component is attached to a node.
	fn on_attachment(&mut self, node_id: NodeId);
	
	/// This function is called when the component is detached from a node.
	fn on_detachment(&mut self, node_id: NodeId);
	
	/// This function is called when the attached-to node is entered.
	fn on_load(&mut self);
	
	/// This function is called when the attached-to node is exited.
	fn on_unload(&mut self);
}

impl PartialEq for dyn Component {
	fn eq(&self, other: &dyn Component) -> bool {
		mopa::Any::get_type_id(self) == mopa::Any::get_type_id(other)
	}
}

impl Eq for dyn Component {
	// empty stub
}

impl std::hash::Hash for dyn Component {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		mopa::Any::get_type_id(self).hash(state);
	}
}


impl std::fmt::Display for dyn Component {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self.get_type_name())
	}
}

// This is 100% necessary until `std::` provides Any for object-traits.
mopafy!(Component);

////////////////////////////////////////////////////////////////////////////////

/// A simple component for testing the backbone.
pub struct DebugComponent;
impl Component for DebugComponent {
	/// Returns an engine internal (no i18n) name for the components type.
	fn get_type_name(&self) -> &'static str {
		"DebugComponent"
	}
	
	/// This function is called when the component is attached to a node.
	fn on_attachment(&mut self, node_id: NodeId) {
		println!("Component attached to node {}.", node_id);
	}
	
	/// This function is called when the component is detached from a node.
	fn on_detachment(&mut self, node_id: NodeId) {
		println!("Component detached from node {}.", node_id);
	}
	
	/// This function is called when the attached-to node is entered.
	fn on_load(&mut self) {
		println!("Component Load");
	}
	
	/// This function is called when the attached-to node is exited.
	fn on_unload(&mut self) {
		println!("Component Unload");
	}
}

////////////////////////////////////////////////////////////////////////////////

/// A component for wrapping objects that can not implement `Component`.
pub struct WrapperComponent<T> {
	pub name: &'static str,
	pub inner: Box<T>
}

impl<T> WrapperComponent<T> {
	pub fn new(name: &'static str, object: T) -> Self {
		Self {
			name,
			inner: Box::new(object)
		}
	}
}

impl<T: 'static> Component for WrapperComponent<T> {
	fn get_type_name(&self) -> &'static str {
		self.name
	}
	
	fn on_attachment(&mut self, _node_id: NodeId) {}
	
	fn on_detachment(&mut self, _node_id: NodeId) {}
	
	fn on_load(&mut self) {}
	
	fn on_unload(&mut self) {}
}

impl<T> std::ops::Deref for WrapperComponent<T> {
	type Target = T;
	
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T> std::ops::DerefMut for WrapperComponent<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
