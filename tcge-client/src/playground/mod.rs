//! Module for prototyping things.

use crate::glfw_context::{GlfwContext, GlInfo};
use crate::{backbone, RenderEvent, TickEvent, KeyEvent, MouseEvent, MouseMoveEvent};
use crate::common::resources;
use crate::blocks;
use crate::render;
use std::rc::Rc;

use legion::prelude::*;

pub mod aabb;

pub mod freecam;
use freecam::Freecam;

pub mod inventory;
use inventory::Inventory;

pub mod sky;
pub mod grid;
pub mod crosshair;

pub mod test_blocks;
use test_blocks::ChunkStorage;
use test_blocks::ChunkRenderManager;
use test_blocks::StaticBlockBakery;
use common::resources::ResourceProvider;

pub fn setup(
	backbone: &mut backbone::Backbone,
	glfw_context: &mut GlfwContext,
	res: &mut resources::Resources,
) {
	info!("Attempting to load ./assets/playground.toml ...");
	let config = match res.res_as_string(&resources::ResourceLocation::from_str("playground.toml")) {
		Ok(config) => match toml::from_str(&config) {
			Ok(config) => if let toml::Value::Table(config) = config {
				info!("Loaded configuration.");
				config
			} else {
				error!("Root is not a table in playground.toml");
				toml::value::Table::new()
			}
			Err(e) => {
				error!("Failed to parse playground.toml: {}", e.to_string());
				toml::value::Table::new()
			}
		},
		Err(_e) => {
			error!("Failed to read playground.toml");
			toml::value::Table::new()
		}
	};
	
	
	
	let entity_universe = Universe::new();
	let mut entity_world = entity_universe.create_world();
	
	let mut freecam = Freecam::new();
	
	if let Some(t) = config.get("freecam") {
		if let toml::Value::Table(t) = t {
			freecam.config(t);
		}
	}
	
	// Create the player entity
	let entity_player = entity_world.insert(
		(),
		vec![
			(freecam, Inventory::default(), )
		]
	)[0];
	
	/*
	debug!("query start");
	for (mut camera,) in <(Write<Freecam>,)>::query().iter(&mut entity_world) {
		debug!("camera {:?}", &camera);
	}
	debug!("query end");
	*/
	
	let blocks = blocks::Blocks::new().to_ref();
	
	let chunks = ChunkStorage::new(&blocks);
	
	let bakery = Rc::new(StaticBlockBakery::new(res, &blocks).expect("StaticBlockBakery initialization must not fail"));
	
	let chunks_renderer = ChunkRenderManager::new(
		&glfw_context.gl,
		res,
		&blocks,
		bakery
	).map_err(|_| {
		error!("Failed to load 'Blocks' material.");
	}).unwrap();
	
	let sky = sky::SkyRenderer::new(&glfw_context.gl, res).map_err(|_| {
		error!("Failed to load 'Blocks' material.");
	}).unwrap();
	
	let grid = grid::GridRenderer::new(&glfw_context.gl, res).map_err(|_| {
		error!("Failed to load 'Grid' material.");
	}).unwrap();
	
	let solid_color_material = render::materials::SolidColorMaterial::new(&glfw_context.gl, &res).map_err(|_| {
		error!("Failed to load 'Grid' material.");
	}).unwrap();
	let solid_color_material = Rc::new(solid_color_material);
	
	let crosshair_2d = crosshair::CrosshairRenderer2D::new(&glfw_context.gl, &solid_color_material);
	let crosshair_3d = crosshair::CrosshairRenderer3D::new(&glfw_context.gl, &solid_color_material);
	
	let playground = Playground {
		entity_universe,
		entity_world,
		entity_player,
		blocks,
		chunks,
		chunks_renderer,
		sky,
		grid,
		crosshair_2d,
		crosshair_3d,
	};
	
	let playground = Box::new(playground);
	let _playground_id = backbone.node_new(
		backbone.root_get_id(),
		"playground",
		Some(playground)
	).unwrap();
}

pub struct Playground {
	entity_universe: legion::world::Universe,
	entity_world: legion::world::World,
	entity_player: legion::entity::Entity,
	blocks: blocks::BlocksRef,
	chunks: ChunkStorage,
	chunks_renderer: ChunkRenderManager,
	sky: sky::SkyRenderer,
	grid: grid::GridRenderer,
	crosshair_2d: crosshair::CrosshairRenderer2D,
	crosshair_3d: crosshair::CrosshairRenderer3D,
}

impl backbone::Handler for Playground {
	fn on_event<'a>(&mut self, event: &mut backbone::Wrapper, context: &mut backbone::Context) {
		let phase = event.get_phase().clone();
		
		if let Some(mouse_move_event) = event.downcast::<MouseMoveEvent>() {
			
			let mut camera  = self.entity_world.get_component_mut::<Freecam>(self.entity_player).expect("player entity freecam component");
			
			camera.update_rotation(
				mouse_move_event.dx as f32,
				mouse_move_event.dy as f32
			);
			
			let mut rc = camera.get_block_raytrace(16.0, 1.0);
			if let Some((_, curr, _)) = self.chunks.raycast(&mut rc) {
				camera.target = Some(curr);
			} else {
				camera.target = None;
			}
			
			return
		}
		
		if let Some(mouse_event) = event.downcast::<MouseEvent>() {
			match mouse_event {
				MouseEvent{button, action: glfw::Action::Press, modifiers: _} => {
					let used_block  = self.entity_world.get_component::<Inventory>(self.entity_player).expect("player entity freecam component").block;
					
					let air = self.blocks
						.get_block_by_name_unchecked("air")
						.get_default_state();
					
					let bedrock = self.blocks
						.get_block_by_name_unchecked("adm")
						.get_default_state();
					
					let camera  = self.entity_world.get_component_mut::<Freecam>(self.entity_player).expect("player entity freecam component");
					
					if ! camera.active {
						return;
					}
					
					let used_block = used_block.unwrap_or(bedrock);
					
					let mut rc = camera.get_block_raytrace(16.0, 1.0);
					
					if let Some((last, curr, _)) = self.chunks.raycast(&mut rc) {
						let t = match button {
							glfw::MouseButtonLeft => {
								Some((&curr, air))
							},
							glfw::MouseButtonRight => {
								Some((&last, used_block))
							},
							_ => None
						};
						
						if let Some((pos, block)) = t {
							self.chunks.set_block(&pos, block);
						}
					}
					
				},
				_ => (),
			};
			
			event.stop();
			return
		}
		
		if let Some(key_event) = event.downcast::<KeyEvent>() {
			match key_event {
				KeyEvent{key: glfw::Key::C, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let mut camera  = self.entity_world.get_component_mut::<Freecam>(self.entity_player).expect("player entity freecam component");
					camera.crane = !camera.crane;
				},
				
				KeyEvent{key: glfw::Key::G, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let mut camera  = self.entity_world.get_component_mut::<Freecam>(self.entity_player).expect("player entity freecam component");
					camera.gravity = !camera.gravity;
				},
				
				KeyEvent{key: glfw::Key::Num1, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let mut inventory  = self.entity_world.get_component_mut::<Inventory>(self.entity_player).expect("player entity inventory component");
					inventory.block = Some(self.blocks.get_block_by_name_unchecked("adm").get_default_state());
				},
				
				KeyEvent{key: glfw::Key::Num2, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let mut inventory  = self.entity_world.get_component_mut::<Inventory>(self.entity_player).expect("player entity inventory component");
					inventory.block = Some(self.blocks.get_block_by_name_unchecked("adm2").get_default_state());
				},
				
				KeyEvent{key: glfw::Key::Num3, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let mut inventory  = self.entity_world.get_component_mut::<Inventory>(self.entity_player).expect("player entity inventory component");
					inventory.block = Some(self.blocks.get_block_by_name_unchecked("adm3").get_default_state());
				},
				
				KeyEvent{key: glfw::Key::Num4, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let mut inventory  = self.entity_world.get_component_mut::<Inventory>(self.entity_player).expect("player entity inventory component");
					inventory.block = Some(self.blocks.get_block_by_name_unchecked("adm4").get_default_state());
				},
				
				KeyEvent{key: glfw::Key::Num5, scancode: _, action: glfw::Action::Press, modifiers: _} => {
					let mut inventory  = self.entity_world.get_component_mut::<Inventory>(self.entity_player).expect("player entity inventory component");
					inventory.block = Some(self.blocks.get_block_by_name_unchecked("adm5").get_default_state());
				},
				
				_ => (),
			}
			
			return
		}
		
		if let Some(tick) = event.downcast::<TickEvent>() {
			let glfw_context = context
				.component_get_mut::<GlfwContext>().ok().unwrap();
			
			let mut camera  = self.entity_world.get_component_mut::<Freecam>(self.entity_player).expect("player entity freecam component");
			
			camera.active = glfw_context.window.get_cursor_mode() == glfw::CursorMode::Disabled;
			camera.update_movement(&glfw_context.window, 1.0 / tick.tps as f32, &self.chunks);
			
			let mut rc = camera.get_block_raytrace(16.0, 1.0);
			if let Some((_, curr, _)) = self.chunks.raycast(&mut rc) {
				camera.target = Some(curr);
			} else {
				camera.target = None;
			}
			return
		}
		
		if let Some(render_event) = event.downcast::<RenderEvent>() {
			let glfw_context = context
				.component_get::<GlfwContext>().ok().unwrap();
			
			if let backbone::Phase::Action = phase {
				self.render_scene(render_event);
			}
			
			if let backbone::Phase::Bubbling = phase {
				let text_renderer = context
					.component_get_mut::<render::text::TextRendererComp>().ok().unwrap();
				
				self.render_hud(render_event, text_renderer, &glfw_context.gl_info);
			}
			
			return
		}
		
		debug!("Playground received {} in {}-phase.", event.event.get_type_name(), event.get_phase());
	}
}

impl Playground {
	
	pub fn render_scene(&mut self, render_event: &RenderEvent) {
		use crate::render::*;
		
		let camera  = self.entity_world.get_component_mut::<Freecam>(self.entity_player).expect("player entity freecam component");
		
		let proj_matrix = camera.get_gl_projection_matrix((render_event.width, render_event.height), render_event.interpolation);
		let skyw_matrix = camera.get_gl_view_matrix(false, render_event.interpolation);
		let view_matrix = camera.get_gl_view_matrix(true, render_event.interpolation);
		let transform = proj_matrix * view_matrix;
		let position = camera.get_position(render_event.interpolation);
		
		self.sky.render(
			&proj_matrix,
			&skyw_matrix,
			&position
		);
		
		self.grid.render(
			&transform,
			&position,
		);
		
		unsafe {
			render_event.gl.Enable(gl::DEPTH_TEST);
		}
		
		self.chunks_renderer.render(&self.chunks, &transform);
		
		if let Some(target) = &camera.target {
			self.crosshair_3d.draw(&transform, target)
		}
	}
	
	pub fn render_hud(&mut self, render_event: &RenderEvent, text: &mut render::text::TextRendererComp, gl_info: &GlInfo) {
		
		unsafe {
			render_event.gl.Disable(gl::DEPTH_TEST);
			render_event.gl.Enable(gl::BLEND);
			render_event.gl.BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
		}
		
		let projection = nalgebra_glm::ortho(
			0.0, render_event.width as f32,
			render_event.height as f32, 0.0,
			-1.0, 1.0
		);
		
		self.crosshair_2d.draw(&projection, render_event.width, render_event.height, 4.0);
		
		text.transform = projection;
		
		let mut y_offset = 2.0;
		
		text.draw_text(&format!("GPU: {}", gl_info.renderer), 16.0, 1.0, y_offset);
		y_offset += 16.0;
		
		text.draw_text(&format!("Blocks: {}", self.chunks.get_approximate_volume()), 16.0, 1.0, y_offset);
		y_offset += 16.0;
		
		let block  = self.entity_world.get_component::<Inventory>(self.entity_player).expect("player entity freecam component").block;
		let camera  = self.entity_world.get_component::<Freecam>(self.entity_player).expect("player entity freecam component");
		
		let cam_pos = camera.get_position(render_event.interpolation);
		text.draw_text(&format!("Position: {:.2}, {:.2}, {:.2}", cam_pos.x, cam_pos.y, cam_pos.z), 16.0, 1.0, y_offset);
		y_offset += 16.0;
		
		if let Some(block_state) = block {
			let block_name = self.blocks.get_block_by_id_unchecked(block_state.id).get_name();
			text.draw_text(&format!("Equipped: {}", block_name), 16.0, 1.0, y_offset);
			y_offset += 16.0;
		}
		
		if let Some(target) = &camera.target {
			if let Some(block_state) = self.chunks.get_block(target) {
				let block_name = self.blocks.get_block_by_id_unchecked(block_state.id).get_name();
				text.draw_text(&format!("Aiming at: {}", block_name), 16.0, 1.0, y_offset);
				y_offset += 16.0;
			}
		}
		
		y_offset += 8.0;
		let profiler = common::profiler::profiler();
		let proftree = profiler.get_passive();
		let mut f_buffer = String::with_capacity(250);
		Self::draw_profiler_node_text(text, proftree, 0, 0, &mut f_buffer, &mut y_offset);
		y_offset += 2.0;
	}
	
	pub fn draw_profiler_node_text(
		text: &mut render::text::TextRendererComp,
		profiler_tree: &common::profiler::ProfilerTree,
		node_id: usize, depth: usize,
		f_buffer: &mut String,
		y_offset: &mut f32
	) {
		let node = &profiler_tree.nodes[node_id];
		
		if node.total_time < 1_000_000 {
			return;
		}
		
		f_buffer.clear();
		use std::fmt::Write;
		write!(f_buffer, "{}: {}", node.name, node.get_time_as_nanosec())
			.ok().expect("Failed to print profiler node.");
		
		text.draw_text(f_buffer, 16.0, 1.0 + (depth as f32 * 24.0), *y_offset);
		*y_offset += 16.0;
		
		for child in node.childs.iter() {
			if profiler_tree.nodes[*child].calls > 0 {
				Self::draw_profiler_node_text(text, profiler_tree, *child, depth + 1, f_buffer, y_offset);
			}
		}
	}
	
}