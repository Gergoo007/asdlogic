use serde::{Deserialize, Serialize};
use wgpu::ShaderModuleDescriptor;

use crate::{canvas::{ElemIndex, Immediates, Vec2}, config};

#[derive(Serialize, Deserialize)]
pub struct CanvasInner {
	pub pan: Vec2,
	pub zoom: f32,
	pub compid: u64,
	pub grab_mouse_offset: Option<(ElemIndex, Vec2)>,
	pub wire_horiz: bool,
	pub debug: bool,
	pub size: Vec2,
		lastmouse: Option<Vec2>,
		#[serde(skip)]
		pipeline: Option<wgpu::RenderPipeline>, // TODO: MaybeUninit ha nagyon lelassítaná
		#[serde(skip)]
	pub wire_draw_cancelled: bool,
}

impl CanvasInner {
	pub fn create_pipeline(&mut self, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) {
		let pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[],
				immediate_size: size_of::<Immediates>() as u32,
			});

		let shader = device.create_shader_module(ShaderModuleDescriptor {
			label: Some("turiplogic background shader"),
			source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string("background.wgsl").expect("background.wgsl not found!").into()),
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

		self.pipeline.replace(pipeline);
	}

	pub fn new(size: &Vec2, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) -> Self {
		let mut s = Self {
			size: Vec2::new(size.x, size.y),
			pan: Vec2::default(),
			zoom: 1.0,
			pipeline: None,
			lastmouse: None,
			compid: 0,
			grab_mouse_offset: None,
			wire_horiz: false,
			debug: false,
			wire_draw_cancelled: false,
		};
		s.create_pipeline(device, surface_desc);
		s
	}

	pub fn record_mouse(&mut self, pos: &Vec2) {
		if self.lastmouse.is_none() { self.lastmouse.replace(self.window_to_canvas(*pos)); }
	}

	pub fn get_mouse(&self) -> Vec2 { self.lastmouse.unwrap() }

	pub fn forget_mouse(&mut self) {
		self.lastmouse.take();
	}

	pub fn canvas_to_window(&self, pos: Vec2) -> Vec2 {
		let center = self.size / 2.0;

		// Ennyi grid koordinátával van elcsúsztatva a canvas
		let pancoord = self.pan / config::GRID_SPACING;

		let grid_spacing_zoom = config::GRID_SPACING * self.zoom;

		return (pos + pancoord) * grid_spacing_zoom + center;
	}

	pub fn window_to_canvas(&self, pos: Vec2) -> Vec2 {
		let center = self.size / 2.0;

		// Ennyi grid koordinátával van elcsúsztatva a canvas
		let pancoord = self.pan / config::GRID_SPACING;

		let grid_spacing_zoom = config::GRID_SPACING * self.zoom;

		return (((pos - center) / grid_spacing_zoom) - pancoord).round();
	}

	pub fn canvas_to_window_size(&self, pos: Vec2, size: Vec2) -> Vec2 {
		(self.canvas_to_window(pos) - self.canvas_to_window(pos + size)).abs()
	}

	pub fn draw_grid(&self, rpass: &mut wgpu::RenderPass) {
		if self.zoom < 0.35 { return; }

		let center = self.size / 2.0;

		let gsz = config::GRID_SPACING * self.zoom;

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

		rpass.set_pipeline(self.pipeline.as_ref().unwrap());
		rpass.set_immediates(0, bytemuck::bytes_of(&turip));
		let cols = self.size.x / gsz + 1.0;
		let rows = self.size.y / gsz + 1.0;
		rpass.draw(0..6, 0..(cols*rows) as u32);
	}
}
