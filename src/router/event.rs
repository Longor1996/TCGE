use super::lens;

/// A event that can be sent trough the router towards various destinations.
/// At the moment the only possible destination is a Lens.
pub trait Event: mopa::Any {
	///	If an event is passive, it can be fired at its destination
	///	regardless of what state the lens is in.
	fn is_passive(&self) -> bool;
}

// This is 100% necessary until `std::` provides Any for object-traits.
mopafy!(Event);

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

/// Wraps an event as it is processed by the [Router].
pub struct Wrapper<'a> {
	#[allow(dead_code)]
	/// The event being processed.
	pub event: &'a mut Event,
	
	// --- State for the event
	phase: Phase,
	
	/// Can the event flow towards its destination?
	can_propagate: bool,
	
	/// Can the event be evaluated by its destination?
	can_default: bool,
	
	/// Can the event flow back towards its source?
	can_bubble: bool,
}

impl<'a> Wrapper<'a> {
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
}

/// Implementation details for event handling.
impl super::Router {
	/// Fires a single `Event` at a single `Lens`.
	pub fn fire_event_at_lens(&mut self, target: &str, event: &mut Event) {
		let lens_id = self.lenses.lenses.iter().position(|lens| { lens.name == target });
		let lens_id = match lens_id {
			Some(x) => x,
			None => return
		};
		
		self.fire_event_at_lens_id(lens_id, event);
	}
	
	/// Actual implementation for `fire_event_at_lens`.
	pub fn fire_event_at_lens_id(&mut self, lens_id: usize, event: &mut Event) {
		let mut nodes = &mut self.nodes;
		let lens = self.lenses.lenses.get_mut(lens_id);
		
		let lens = match lens {
			Some(x) => x,
			None => return
		};
		
		let lens_handler = self.lenses.handlers.get_mut(lens_id);
		let lens_handler = match lens_handler {
			Some(x) => x,
			None => return
		};
		
		// A lens can only receive an event if inactive or the event is PASSIVE.
		if !event.is_passive() {
			if lens.state != lens::State::Idle {
				return
			}
		}
		
		// A lens without path cannot receive events
		if lens.path.len() == 0 {
			return;
		}
		
		// Holder for event state.
		let mut event_wrapper = Wrapper {
			event,
			
			// Initial State
			phase: Phase::Creation,
			can_propagate: true,
			can_default: true,
			can_bubble: true,
		};
		
		// --- Event Propagation
		event_wrapper.phase = Phase::Propagation;
		for node_id in lens.path.iter() {
			let (node, comps)
				= nodes.get_mut_node_with_comps_by_id(*node_id);
			
			node.map(|node| {
				node.on_event(comps, &mut event_wrapper);
			});
			
			if !event_wrapper.can_propagate {
				break;
			}
		}
		
		// --- Event Action
		let new_state = if event_wrapper.can_default {
			event_wrapper.phase = Phase::Action;
			
			(*lens_handler).on_event(&mut event_wrapper, lens, &mut nodes)
		} else {
			lens::State::Idle
		};
		
		// --- Event Bubbling
		if event_wrapper.can_bubble {
			event_wrapper.phase = Phase::Bubbling;
			for node_id in lens.path.iter().rev() {
				
				let (node, comps)
					= nodes.get_mut_node_with_comps_by_id(*node_id);
				
				node.map(|node| {
					node.on_event(comps, &mut event_wrapper);
				});
				
				if !event_wrapper.can_bubble {
					break;
				}
			}
		}
		
		if lens.state != lens::State::Idle {
			// Don't start a new action if one is already running!
			return
		}
		
		// Swap in the action, kicking off whatever action the lens wants...
		lens.state = new_state
	}
	
	/// Directly trigger an event for a lens, completely ignoring the normal event flow.
	pub fn trigger_event_at_lens_id(&mut self, lens_id: usize, event: &mut Event) -> bool {
		let mut nodes = &mut self.nodes;
		let lens = self.lenses.lenses.get_mut(lens_id);
		
		let lens = match lens {
			Some(x) => x,
			None => return false
		};
		
		let lens_handler = self.lenses.handlers.get_mut(lens_id);
		let lens_handler = match lens_handler {
			Some(x) => x,
			None => return false
		};
		
		// A lens can only receive an event if inactive or the event is PASSIVE.
		if !event.is_passive() {
			if lens.state != lens::State::Idle {
				return false
			}
		}
		
		let mut wrapper = Wrapper {
			event,
			phase: Phase::Action,
			can_propagate: false,
			can_default: true,
			can_bubble: false,
		};
		
		lens_handler.on_event(&mut wrapper, &lens, &mut nodes);
		true
	}
	
	/// Directly trigger an event for a node, completely ignoring the normal event flow.
	pub fn trigger_event_at_node_id(&mut self, node_id: usize, event: &mut Event) -> bool {
		match self.nodes.nodes.get_mut(&node_id) {
			Some(node) => {
				let mut wrapper = Wrapper {
					event,
					phase: Phase::Action,
					can_propagate: false,
					can_default: true,
					can_bubble: false,
				};
				
				let mut comps = self.nodes.comps.comps.get_mut(&node_id);
				node.on_event(comps, &mut wrapper);
				true
			}
			None => false
		}
	}
	
	/// Fires a single `Event` at a single `Node`, given a path.
	#[allow(unused)]
	pub fn fire_event_at_node(&mut self, path: &str, event: &mut Event) {
		/*
		let mut path_offset = 0;
		let mut src_path = vec![];
		let mut bubble_path = vec![];
		
		loop {
			
			let step = Router::path_next(
				&self.nodes.nodes,
				path,
				&mut path_offset,
				src_path.as_slice()
			);
			
			// TODO: Implement firing of events at nodes.
		}
		*/
	}
}
