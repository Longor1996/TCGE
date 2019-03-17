use core::borrow::{BorrowMut, Borrow};

pub fn new_router() -> Router {
	let mut router = Router {
		lenses: vec![],
		nodes: vec![],
	};
	
	router.nodes.push(Some(Node {
		id: 0,
		parent: None,
		name: "root".to_string(),
	}));
	
	return router;
}

/******************************************************************************/

pub struct Router {
	lenses: Vec<Lens>,
	nodes: Vec<Option<Node>>,
}

impl Router {
	//
	
	pub fn new_lens(&mut self, name: &str, constructor: &Fn(&mut Lens)) {
		let mut lens = Lens {
			name: name.to_string(),
			path: vec![],
			handler: Box::new(NULL_HANDLER),
			state: LensState::Idle,
		};
		
		constructor(&mut lens);
		self.lenses.push(lens);
	}
	
	pub fn update(&mut self) -> bool {
		let mut events: Vec<(usize, Box<Event>)> = vec![];
		
		for (pos, lens) in self.lenses.iter_mut().enumerate() {
			// Ignore all idle and destroying lenses
			if lens.state == LensState::Idle || lens.state == LensState::Destruction {
				continue
			}
			
			if lens.path.is_empty() {
				// All lenses must be at least at root-level
				lens.state = LensState::Moving("/".to_string());
			}
			
			// TODO: Actually implement routing...
			let mut state = lens.state.clone();
			match state {
				_ => {},
			}
			
			// let mut finish_event = LensMoveEvent::Finished;
			// events.push((pos, Box::new(finish_event)));
			
			lens.state = LensState::Idle;
		}
		
		while let Some((pos, mut event)) = events.pop() {
			self.fire_event_at_lens_id(
				pos,
				(*event).borrow_mut()
			);
		}
		
		// Remove all lenses that want to self-destruct.
		self.lenses.retain(
			|lens| lens.state != LensState::Destruction
		);
		
		return self.lenses.is_empty()
	}
}

/******************************************************************************/

pub struct Node {
	pub parent: Option<usize>,
	pub name: String,
	pub id: usize,
}

impl Node {
	pub fn on_event(&mut self, event: &mut EventWrapper) {
		// TODO: Walk trough event-listeners/components...
	}
}

/******************************************************************************/

pub struct Lens {
	pub name: String,
	pub path: Vec<usize>,
	pub handler: Box<LensHandler>,
	pub state: LensState,
}

pub trait LensHandler {
	/* Called when the lens receives an event. */
	fn on_event(&mut self, event: &mut EventWrapper) -> LensState;
}

/* // TODO: Correctly implement this once https://areweasyncyet.rs/ is ready.
impl LensHandler {
	pub fn from_fn<'a, FnType: 'a>(handler_fn: FnType) -> Box<LensHandler + 'a>
		where FnType: Fn(&mut LensHandler, &mut EventWrapper) -> LensAction
	{
		struct FnLensHandler<'a> {
			handler_fn: Box<Fn(&mut LensHandler, &mut EventWrapper) -> LensAction + 'a>
		}
		
		impl<'a> LensHandler for FnLensHandler<'a> {
			fn on_event(&mut self, event: &mut EventWrapper) -> LensAction {
				(self.handler_fn)(self, event)
			}
		}
		
		Box::new(FnLensHandler {
			handler_fn: Box::new(handler_fn)
		})
	}
}
*/

const NULL_HANDLER: NullLensHandler = NullLensHandler {};
pub struct NullLensHandler {}
impl LensHandler for NullLensHandler {
	fn on_event(&mut self, event: &mut EventWrapper) -> LensState {
		LensState::Idle
	}
}

#[derive(Clone)]
pub enum LensState {
	Idle,
	Moving(String),
	Destruction,
}

impl PartialEq for LensState {
	fn eq(&self, other: &LensState) -> bool {
		std::mem::discriminant(self) == std::mem::discriminant(other)
	}
}

enum LensMoveEvent {
	Finished,
	Aborted
}

impl Event for LensMoveEvent {
	fn is_passive(&self) -> bool { false }
}

/******************************************************************************/

/// A event that can be sent trough the router towards various destinations.
/// At the moment the only possible destination is a Lens.
pub trait Event: mopa::Any {
	///	If an event is passive, it can be fired at its destination
	///	regardless of what state the lens is in.
	fn is_passive(&self) -> bool;
}

// This is 100% necessary until `std::` provides Any for object-traits.
mopafy!(Event);

pub enum EventPhase {
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
pub struct EventWrapper<'a> {
	#[allow(dead_code)]
	/// The event being processed.
	pub event: &'a mut Event,
	
	// --- State for the event
	phase: EventPhase,
	
	/// Can the event flow towards its destination?
	can_propagate: bool,
	
	/// Can the event be evaluated by its destination?
	can_default: bool,
	
	/// Can the event flow back towards its source?
	can_bubble: bool,
}

impl<'a> EventWrapper<'a> {
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
impl Router {
	/// Fires a single `Event` at a single `Lens`.
	pub fn fire_event_at_lens(&mut self, target: &str, event: &mut Event) {
		let lens_id = self.lenses.iter().position(|lens| { lens.name == target });
		let lens_id = match lens_id {
			Some(x) => x,
			None => return
		};
		
		self.fire_event_at_lens_id(lens_id, event);
	}
	
	/// Actual implementation for `fire_event_at_lens`.
	fn fire_event_at_lens_id(&mut self, target_id: usize, event: &mut Event) {
		let lens = self.lenses.get_mut(target_id);
		let lens = match lens {
			Some(x) => x,
			None => return
		};
		
		// A lens can only receive an event if inactive or the event is PASSIVE.
		if !event.is_passive() {
			if lens.state != LensState::Idle {
				return
			}
		}
		
		// A lens without path cannot receive events
		/*
		if lens.path.len() == 0 {
			return;
		}
		*/
		
		let mut event_wrapper = EventWrapper {
			event,
			
			// Initial State
			phase: EventPhase::Creation,
			can_propagate: true,
			can_default: true,
			can_bubble: true,
		};
		
		event_wrapper.phase = EventPhase::Propagation;
		for node_id in lens.path.iter() {
			self.nodes[*node_id].as_mut().map(|n|
				n.on_event(&mut event_wrapper)
			);
			
			if !event_wrapper.can_propagate {
				break;
			}
		}
		
		let new_state = if event_wrapper.can_default {
			event_wrapper.phase = EventPhase::Action;
			(*lens.handler).on_event(&mut event_wrapper)
		} else {
			LensState::Idle
		};
		
		if event_wrapper.can_bubble {
			event_wrapper.phase = EventPhase::Bubbling;
			for node_id in lens.path.iter().rev() {
				self.nodes[*node_id].as_mut().map(|n|
					n.on_event(&mut event_wrapper)
				);
				
				if !event_wrapper.can_bubble {
					break;
				}
			}
		}
		
		if lens.state != LensState::Idle {
			// Do start a new action if one is already running
			return
		}
		
		// Swap in the action, kicking off whatever action the lens wants...
		lens.state = new_state
	}
}
