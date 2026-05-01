use bytemuck::{Pod, Zeroable};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Queue, RenderPass, RenderPipeline, ShaderModuleDescriptor, VertexAttribute, VertexBufferLayout};

use crate::canvas::{CompStorage, Vec2, component::ShapeElement, inner::CanvasInner, nodes::NodeStorage, wires::Wires};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
struct LineInput {
	s: [f32; 2],
	e: [f32; 2],
	c: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RenderImmediates {
	pan: [f32; 2],
	wsize: [f32; 2],
	zoom: f32,
}

pub struct CanvasRenderer {
	pipeline: Option<RenderPipeline>,
	linebuf: Buffer,
	linebuf_local: Vec<LineInput>,
}

const LINE_CAPACITY: u64 = 300000;

impl CanvasRenderer {
	pub fn create_pipeline(&mut self, device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) {
		let pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Line render pipeline layout"),
				bind_group_layouts: &[],
				immediate_size: size_of::<RenderImmediates>() as u32,
			});

		let shader = device.create_shader_module(ShaderModuleDescriptor {
			label: Some("turiplogic line shader"),
			source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string("lines.wgsl").expect("lines.wgsl not found!").into()),
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: Some("vs_main"),
				buffers: &[
					VertexBufferLayout {
						array_stride: std::mem::size_of::<LineInput>() as wgpu::BufferAddress,
						step_mode: wgpu::VertexStepMode::Instance,
						attributes: &[
							VertexAttribute {
								format: wgpu::VertexFormat::Float32x2,
								offset: 0,
								shader_location: 0,
							},
							VertexAttribute {
								format: wgpu::VertexFormat::Float32x2,
								offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
								shader_location: 1,
							},
							VertexAttribute {
								format: wgpu::VertexFormat::Uint32,
								offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress * 2,
								shader_location: 2,
							}
						],
					}
				],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fs_main"),
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_desc.format,
					blend: Some(wgpu::BlendState::ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: None, // TODO
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

	fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
		if t == 0.0 { return p0; }

		let q0 = p0.lerp(p1, t);
		let q1 = p1.lerp(p2, t);
		let q2 = p2.lerp(p3, t);

		let r0 = q0.lerp(q1, t);
		let r1 = q1.lerp(q2, t);

		r0.lerp(r1, t)
	}

	pub fn new(device: &wgpu::Device, surface_desc: &wgpu::SurfaceConfiguration) -> Self {
		let linebuf = device.create_buffer(&BufferDescriptor {
			label: None,
			mapped_at_creation: false,
			size: size_of::<LineInput>() as u64 * LINE_CAPACITY, // 80 MiB
			usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
		});

		let mut s = Self {
			pipeline: None,
			linebuf,
			linebuf_local: Vec::with_capacity(LINE_CAPACITY as usize),
		};

		s.create_pipeline(device, surface_desc);

		s
	}

	const SELECTCOLOR: u32 = 0xffaaaaaa;

	pub fn regenerate_buffer(&mut self, wires: &Wires, nodes: &NodeStorage, comps: &CompStorage, zoom: f32, queue: &mut Queue) {
		self.linebuf_local.clear();
		for (_, c) in comps {
			if c.selected {
				let hitbox = c.kind.hitbox();
				let padding = 0.5;

				let negx = -padding + c.pos.x;
				let posx = padding + c.pos.x + hitbox.x;

				let negy = -padding + c.pos.y;
				let posy = padding + c.pos.y + hitbox.y;

				self.linebuf_local.push(LineInput {
					s: [ negx, negy ],
					e: [ posx, negy ],
					c: Self::SELECTCOLOR,
				});

				self.linebuf_local.push(LineInput {
					s: [ posx, negy ],
					e: [ posx, posy ],
					c: Self::SELECTCOLOR,
				});

				self.linebuf_local.push(LineInput {
					s: [ posx, posy ],
					e: [ negx, posy ],
					c: Self::SELECTCOLOR,
				});

				self.linebuf_local.push(LineInput {
					s: [ negx, posy ],
					e: [ negx, negy ],
					c: Self::SELECTCOLOR,
				});
			}

			for s in c.kind.shape() {
				match s {
					ShapeElement::Line(vec2, vec3) => {
						self.linebuf_local.push(LineInput {
							s: (c.pos + vec2).into(),
							e: (c.pos + vec3).into(),
							c: 0xffffffff,
						});
					},
					ShapeElement::Bezier(vec2, vec3, vec4, vec5) => {
						let segments: u32 = match zoom {
							0.0..0.1 => 2,
							0.1..0.35 => 4,
							0.35..1.0 => 16,
							1.0.. => 32,
							_ => unreachable!("Invalid zoom value")
						};
						for s in 1..=segments {
							let mut v0 = Self::cubic_bezier(c.pos + vec2, c.pos + vec3, c.pos + vec4, c.pos + vec5, (s-1) as f32 / segments as f32);
							let mut v1 = Self::cubic_bezier(c.pos + vec2, c.pos + vec3, c.pos + vec4, c.pos + vec5, s as f32 / segments as f32);
							let stitching_fix = 0.01;
							v0 -= stitching_fix * (v1 - v0).normalize();
							v1 -= stitching_fix * (v0 - v1).normalize();
							self.linebuf_local.push(
								LineInput {
									s: v0.into(),
									e: v1.into(),
									c: 0xffffffff,
								}
							);
						}

					},
					ShapeElement::Circle(_) => {
						// builder.add_circle(point(vec2.x, vec2.y), 10.0, lyon::path::Winding::Positive);
					},
					ShapeElement::Nop => {},
				}
			}
		}
		for (_, w) in &wires.wires {
			if w.selected {
				let padding = 0.5;

				let negx = -padding + w.start.x.min(w.end.x);
				let posx = padding + w.end.x.max(w.start.x);

				let negy = -padding + w.start.y.min(w.end.y);
				let posy = padding + w.end.y.max(w.start.y);

				self.linebuf_local.push(LineInput {
					s: [ negx, negy ],
					e: [ posx, negy ],
					c: Self::SELECTCOLOR,
				});

				self.linebuf_local.push(LineInput {
					s: [ posx, negy ],
					e: [ posx, posy ],
					c: Self::SELECTCOLOR,
				});

				self.linebuf_local.push(LineInput {
					s: [ posx, posy ],
					e: [ negx, posy ],
					c: Self::SELECTCOLOR,
				});

				self.linebuf_local.push(LineInput {
					s: [ negx, posy ],
					e: [ negx, negy ],
					c: Self::SELECTCOLOR,
				});
			}

			self.linebuf_local.push(LineInput {
				s: w.start.into(),
				e: w.end.into(),
				c: nodes[w.startnode.unwrap()].logic_lvl.to_color()
			});
		}
		queue.write_buffer(&self.linebuf, 0, bytemuck::cast_slice(&self.linebuf_local));
	}

	pub fn render(&self, rpass: &mut RenderPass, canvas: &CanvasInner, wsize: [f32; 2]) {
		rpass.set_pipeline(self.pipeline.as_ref().unwrap());
		rpass.set_immediates(0, bytemuck::bytes_of(&RenderImmediates {
			pan: canvas.pan.into(),
			zoom: canvas.zoom,
			wsize,
		}));
		rpass.set_vertex_buffer(0, self.linebuf.slice(..));
		rpass.draw(0..6, 0..self.linebuf_local.len() as u32);
	}

	pub fn linebuf_len(&self) -> usize { self.linebuf_local.len() }
}
