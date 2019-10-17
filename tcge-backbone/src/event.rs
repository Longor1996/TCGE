use super::NodeId;
use super::State;

/// Collection type for `Handler` instances, attached to `Node` instances by id.
pub type Handlers = super::FxHashMap<NodeId, Box<dyn Handler>>;

// Implementation details regarding nodes.
impl super::Backbone {
	
	pub fn fire_event(&mut self, event: &mut dyn Event) {
		
		// Non-Passive events can only be fired if the backbone is idle.
		if ! event.is_passive() {
			if self.state != State::Idle {
				return
			}
		}
		
		let mut event = Wrapper::new(event);
		self.fire_event_impl(&mut event, 0);
		
		if event.new_state == State::Idle {
			return
		}
		
		if self.state.can_replace() {
			self.state = event.new_state;
		}
	}
	
	fn fire_event_impl(&mut self, event: &mut Wrapper, depth: usize) {
		
		// If the backbone path is empty, we cant do anything.
		if self.path.is_empty() {
			return
		}
		
		// Due to the above call, this line can't fail.
		let target_id = self.location_get_node().unwrap();
		
		// --- Event Propagation
		self.fire_event_propagate(event, target_id);
		
		// --- Event Action
		self.fire_event_action(event, target_id);
		
		// --- Event Bubbling
		self.fire_event_bubbling(event, target_id);
		
		self.fire_event_next(event, depth);
	}
	
	fn fire_event_propagate(&mut self, mut event: &mut Wrapper, target_id: NodeId) {
		if !event.can_propagate {
			return
		}
		
		event.phase = Phase::Propagation;
		
		for node_id in self.path.iter() {
			let handler = self.handlers.get_mut(node_id);
			
			let mut context = Context {
				target: target_id,
				current: *node_id,
				nodes: &self.nodes,
				comps: &mut self.comps,
				path: &self.path,
				path_str: &self.path_str.as_str(),
			};
			
			handler.map(|handler| {
				handler.on_event(&mut event, &mut context);
			});
			
			if !event.can_propagate {
				break;
			}
		}
	}
	
	fn fire_event_action(&mut self, mut event: &mut Wrapper, target_id: NodeId) {
		if ! event.can_default {
			return
		}
		
		event.phase = Phase::Action;
		
		if let Some(handler) = self.handlers.get_mut(&target_id) {
			let mut context = Context {
				target: target_id,
				current: target_id,
				nodes: &self.nodes,
				comps: &mut self.comps,
				path: &self.path,
				path_str: &self.path_str.as_str(),
			};
			
			handler.on_event(
				&mut event,
				&mut context
			);
		}
	}
	
	fn fire_event_bubbling(&mut self, mut event: &mut Wrapper, target_id: NodeId) {
		if !event.can_bubble {
			return
		}
		
		event.phase = Phase::Bubbling;
		
		for node_id in self.path.iter().rev() {
			let handler = self.handlers.get_mut(node_id);
			
			let mut context = Context {
				target: target_id,
				current: *node_id,
				nodes: &self.nodes,
				comps: &mut self.comps,
				path: &self.path,
				path_str: &self.path_str.as_str(),
			};
			
			handler.map(|handler| {
				handler.on_event(&mut event, &mut context);
			});
			
			if !event.can_bubble {
				break;
			}
		}
	}
	
	fn fire_event_next(&mut self, mut event: &mut Wrapper, depth: usize) {
		// This allows handlers to fire events in response to events.
		while let State::Fire(_) = event.new_state {
			if depth > 10 {
				return
			}
			
			if let State::Fire(mut sub_event) = std::mem::replace(&mut event.new_state, State::Idle) {
				let mut sub_event = Wrapper::new(&mut *sub_event);
				
				// WARNING: This can potentially cause recursion to occur.
				self.fire_event_impl(&mut sub_event, depth + 1);
				
				if sub_event.new_state != State::Idle {
					event.new_state = sub_event.new_state;
				}
			}
		}
	}
}

////////////////////////////////////////////////////////////////////////////////

/// A limited version of the backbone, that only allows mutating components,
/// intended for event processing.
pub struct Context<'a> {
	pub target: NodeId,
	pub current: NodeId,
	pub nodes: &'a super::Nodes,
	pub comps: &'a mut super::Comps,
	pub path: &'a super::Path,
	pub path_str: &'a str,
}

////////////////////////////////////////////////////////////////////////////////

/// An event that can be fired into the backbone.
pub trait Event: mopa::Any {
	///	If an event is passive, it can be fired
	///	regardless of what state the backbone is in.
	fn is_passive(&self) -> bool {false}
	
	fn get_type_name(&self) -> &'static str;
}

// This is 100% necessary until `std::` provides Any for object-traits.
mopafy!(Event);

/// Represents the phase (or state) of an event as it's being processed.
#[derive(Eq, Hash, Clone, Copy, Debug)]
pub enum Phase {
	/// The event is being wrapped in a `EventWrapper`.
	Creation,
	
	/// The event is flowing towards its destination.
	Propagation,
	
	/// The event is being evaluated by its destination.
	Action,
	
	/// The event is flowing back towards its source.
	Bubbling
}

impl PartialEq for Phase {
	fn eq(&self, other: &Phase) -> bool {
		std::mem::discriminant(self) == std::mem::discriminant(other)
	}
}

impl std::fmt::Display for Phase {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Phase::Creation    => write!(fmt, "Creation"),
			Phase::Propagation => write!(fmt, "Propagation"),
			Phase::Action      => write!(fmt, "Action"),
			Phase::Bubbling    => write!(fmt, "Bubbling"),
		}
	}
}

/// Wraps an event as it is processed by the backbone.
pub struct Wrapper<'a> {
	#[allow(dead_code)]
	/// The event being processed.
	pub event: &'a mut dyn Event,
	
	// --- State for the event
	phase: Phase,
	
	/// Can the event flow towards its destination?
	can_propagate: bool,
	
	/// Can the event be evaluated by its destination?
	can_default: bool,
	
	/// Can the event flow back towards its source?
	can_bubble: bool,
	
	/// What state should the backbone jump to afterwards?
	new_state: State,
}

impl<'a> Wrapper<'a> {
	
	pub fn new(event: &'a mut dyn Event) -> Self {
		Self {
			event,
			
			// Initial State
			phase: Phase::Creation,
			can_propagate: true,
			can_default: true,
			can_bubble: true,
			
			new_state: State::Idle,
		}
	}
	
	/// Downcast the wrapped event into the given type, if possible.
	pub fn downcast<E: Event>(&mut self) -> Option<&E> {
		self.event.downcast_ref()
	}
	
	/// Downcast the wrapped event into the given type, if possible.
	pub fn downcast_mut<E: Event>(&mut self) -> Option<&mut E> {
		self.event.downcast_mut()
	}
	
	/// Prevents the event from being evaluated by its destination.
	pub fn prevent_default(&mut self) {
		self.can_default = false;
	}
	
	/// Stops the flow of the event toward its destination.
	pub fn stop_propagation(&mut self) {
		self.can_propagate = false;
	}
	
	/// Stops the flow of the event back towards its source.
	pub fn stop_bubbling(&mut self) {
		self.can_bubble = false;
	}
	
	pub fn get_phase(&self) -> Phase {
		self.phase
	}
	
	pub fn new_state(&mut self, state: State) {
		self.new_state = state;
	}
	
	pub fn stop(&mut self) {
		self.can_propagate = false;
		self.can_default = false;
		self.can_bubble = false;
	}
}

/// A node-handler is effectively the 'brain' of a node.
/// All user-logic and -state for a node is owned by the handler.
pub trait Handler {
	/// Called when the node receives an event.
	/// Can return a new state for the backbone.
	fn on_event<'a>(
		&mut self,
		event: &mut Wrapper,
		context: &mut Context,
	);
}
