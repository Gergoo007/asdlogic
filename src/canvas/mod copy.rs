use std::{any::Any, fmt::format, fs};

use strum::{EnumMessage, IntoEnumIterator};
use wgpu::ShaderModuleDescriptor;

use crate::canvas::component::{Component, Gate, GateKind};

mod component;

pub type Vec2 = glam::Vec2;

pub struct Canvas {
	size: Vec2,
	pan: Vec2,
	zoom: f32,
	comps: Vec<Component>,
	pipeline: wgpu::RenderPipeline,
	lastmouse: Option<Vec2>,
	compid: u64,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pos: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Immediates {
	start: [f32; 2],
	step: [f32; 2],
	num_cols: u32,
	zoom: f32,
}

impl Canvas {
	const GRID_SPACING: f32 = 16.0;

	pub fn new(size: &Vec2, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) -> Self {
		let pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[],
				immediate_size: size_of::<Immediates>() as u32,
			});

		let shader = device.create_shader_module(ShaderModuleDescriptor {
			label: Some("turiplogic vertex shader"),
			source: wgpu::ShaderSource::Wgsl(fs::read_to_string("shader.wgsl").expect("shader.wgsl not found!").into()),
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: Some("vs_main"),
				buffers: &[],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fs_main"),
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_desc.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview_mask: None,
			cache: None,
		});

		let comps = Vec::new();

		Self {
			size: Vec2::new(size.x, size.y),
			pan: Vec2::default(),
			zoom: 1.0,
			comps,
			pipeline,
			lastmouse: None,
			compid: 0,
		}
	}

	fn record_mouse(&mut self, pos: &Vec2) {
		if self.lastmouse.is_none() { self.lastmouse.replace(self.window_to_canvas(*pos)); }
	}

	fn get_mouse(&self) -> Vec2 { self.lastmouse.unwrap() }

	fn forget_mouse(&mut self) {
		self.lastmouse.take();
	}

	fn canvas_to_window(&self, pos: Vec2) -> Vec2 {
		let center = self.size / 2.0;

		// Ennyi grid koordinátával van elcsúsztatva a canvas
		let pancoord = self.pan / Canvas::GRID_SPACING;

		let grid_spacing_zoom = Canvas::GRID_SPACING * self.zoom;

		return (pos + pancoord) * grid_spacing_zoom + center;
	}

	fn window_to_canvas(&self, pos: Vec2) -> Vec2 {
		let center = self.size / 2.0;

		// Ennyivel van elcsúszva a grid, tehát a pontok amikhez a koordinátát snappelni kell
		let panoffset = self.pan % Canvas::GRID_SPACING;

		// Ennyi grid koordinátával van elcsúsztatva a canvas
		let pancoord = self.pan / Canvas::GRID_SPACING;

		let grid_spacing_zoom = Canvas::GRID_SPACING * self.zoom;

		return ((((pos - center) - panoffset) / grid_spacing_zoom) - pancoord).round();
	}

	fn canvas_to_window_size(&self, pos: Vec2, size: Vec2) -> Vec2 {
		(self.canvas_to_window(pos) - self.canvas_to_window(pos + size)).abs()
	}

	pub fn draw(&mut self, ui: &imgui::Ui) {
		let io = ui.io();

		// Zoom kezelése
		self.zoom = self.zoom + ((io.mouse_wheel * 1024.0).round() / 1024.0) / 20.0 * self.zoom;

		// Pan
		if io.mouse_down[imgui::MouseButton::Middle as usize] {
			self.pan += Vec2::from(io.mouse_delta) / self.zoom;
		}

		ui.text(format!("FPS: {:.2} ({:.2} ms)", 1.0 / io.delta_time, io.delta_time));
		ui.text(format!("zoom: {}", self.zoom));

		if let Some(_) = ui.begin_popup_context_window() {
			self.record_mouse(&io.mouse_pos.into());

			if let Some(_) = ui.begin_menu("Spawn") {
				if let Some(_) = ui.begin_menu("Logic Gate") {
					for asd in GateKind::iter() {
						if ui.menu_item(format!("{}", asd.get_message().unwrap())) {
							self.comps.push(
								Component::Gate(Gate::new(asd, self.get_mouse()))
							);
							self.compid += 1;
							self.forget_mouse();
						}
					}
				}
			}
		}

		for c in &mut self.comps {
			c.draw(self, ui);
		}
	}

	pub fn draw_grid(&self, rpass: &mut wgpu::RenderPass) {
		if self.zoom < 0.35 { return; }

		let center = self.size / 2.0;

		let gsz = Canvas::GRID_SPACING * self.zoom;

		let mut remainder = (((center / gsz) - (center / gsz).floor()) * gsz
							+ ((self.pan * self.zoom) % gsz) + gsz) % gsz;

		// Normalizálás: Ablakkoordináták -> NDC
		remainder = 2.0 * remainder / self.size - 1.0;
		remainder.y = -remainder.y;
		let step = gsz * 2.0 / self.size;

		let num_cols = (self.size.x / gsz) as u32 + 1;

		let turip = Immediates {
			start: remainder.into(),
			step: step.into(),
			num_cols,
			zoom: self.zoom,
		};

		rpass.set_pipeline(&self.pipeline);
		rpass.set_immediates(0, bytemuck::bytes_of(&turip));
		let cols = self.size.x / gsz + 1.0;
		let rows = self.size.y / gsz + 1.0;
		rpass.draw(0..6, 0..(cols*rows) as u32);
	}
}
