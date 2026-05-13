use glam::Vec2;
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use pollster::block_on;
use wgpu::{FeaturesWebGPU, InstanceFlags};
use std::{fs, sync::Arc, time::Instant};
use winit::{
	application::ApplicationHandler,
	dpi::LogicalSize,
	event::{Event, WindowEvent},
	event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
	window::Window,
};
use clap::Parser;

mod config;
mod canvas;

struct ImguiState {
	context: imgui::Context,
	platform: WinitPlatform,
	renderer: Renderer,
	clear_color: wgpu::Color,
	last_frame: Instant,
	last_cursor: Option<MouseCursor>,
}

struct AppWindow {
	device: wgpu::Device,
	queue: wgpu::Queue,
	window: Arc<Window>,
	surface_desc: wgpu::SurfaceConfiguration,
	surface: wgpu::Surface<'static>,
	hidpi_factor: f64,
	imgui: Option<ImguiState>,
	canvas: canvas::Canvas,
}

#[derive(Default)]
struct App {
	window: Option<AppWindow>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	#[arg(short, long, default_value_t = str::to_string("vulkan"))]
	backend: String,

	#[arg(short, long, default_value_t = false)]
	validation: bool,
}

impl AppWindow {
	fn setup_gpu(event_loop: &ActiveEventLoop) -> Self {
		let args = Args::parse();
		let f = if args.validation { InstanceFlags::debugging() } else { InstanceFlags::empty() };

		let b = match args.backend.as_str() {
			"vulkan" => wgpu::Backends::VULKAN,
			"dx12" => wgpu::Backends::DX12,
			"metal" => wgpu::Backends::METAL,
			_ => panic!("Invalid backend ({})! Valid backends: vulkan, dx12, metal", args.backend),
		};

		let instance =
			wgpu::Instance::new(wgpu::InstanceDescriptor {
				backends: b,
				flags: f,
				..wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
					event_loop.owned_display_handle(),
				))
			});

		let init_size = LogicalSize::new(1280.0, 720.0);
		let window = {
			let attributes = Window::default_attributes()
				.with_inner_size(init_size)
				.with_title(format!("turiplogic v0.000000000000004"));
			Arc::new(event_loop.create_window(attributes).unwrap())
		};

		let size = window.inner_size();
		let hidpi_factor = window.scale_factor();
		let surface = instance.create_surface(window.clone()).unwrap();

		let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::HighPerformance,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		}))
		.unwrap();

		let mut cfg = wgpu::DeviceDescriptor::default();
		cfg.required_features.features_webgpu.set(FeaturesWebGPU::IMMEDIATES, true);
		cfg.required_limits.max_immediate_size = 32;

		let (device, queue) =
			block_on(adapter.request_device(&cfg)).unwrap();

		// Set up swap chain
		let surface_desc = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::AutoNoVsync,
			desired_maximum_frame_latency: 4,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
		};

		surface.configure(&device, &surface_desc);

		let imgui = None;

		let project = canvas::Canvas::new(&Vec2::new(surface_desc.width as f32, surface_desc.height as f32), &device, &surface_desc);

		Self {
			device,
			queue,
			window,
			surface_desc,
			surface,
			hidpi_factor,
			imgui,
			canvas: project
		}
	}

	fn setup_imgui(&mut self) {
		let mut context = imgui::Context::create();
		let mut platform = imgui_winit_support::WinitPlatform::new(&mut context);
		platform.attach_window(
			context.io_mut(),
			&self.window,
			imgui_winit_support::HiDpiMode::Default,
		);
		context.set_ini_filename(None);

		let font_size = (20.0 * self.hidpi_factor) as f32;
		context.io_mut().font_global_scale = (1.0 / self.hidpi_factor) as f32;

		let fontfile = fs::read("font.ttf").expect("Font file not found! Make sure font.ttf is in the current working directory!");
		context.fonts().add_font(&[imgui::FontSource::TtfData { data: &fontfile, size_pixels: font_size, config: None }]);

		//
		// Set up dear imgui wgpu renderer
		//
		let clear_color = wgpu::Color {
			r: 0.01,
			g: 0.01,
			b: 0.01,
			a: 1.0,
		};

		let renderer_config = RendererConfig {
			texture_format: self.surface_desc.format,
			..Default::default()
		};

		let renderer = Renderer::new(&mut context, &self.device, &self.queue, renderer_config);
		let last_frame = Instant::now();
		let last_cursor = None;

		self.imgui = Some(ImguiState {
			context,
			platform,
			renderer,
			clear_color,
			last_frame,
			last_cursor,
		})
	}

	fn new(event_loop: &ActiveEventLoop) -> Self {
		let mut window = Self::setup_gpu(event_loop);
		window.setup_imgui();
		window
	}
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		self.window = Some(AppWindow::new(event_loop));
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: winit::window::WindowId,
		event: WindowEvent,
	) {
		let window = self.window.as_mut().unwrap();
		let imgui = window.imgui.as_mut().unwrap();

		match &event {
			WindowEvent::Resized(size) => {
				if size.width != 0 && size.height != 0 {
					window.surface_desc.width = size.width;
					window.surface_desc.height = size.height;
					window
						.surface
						.configure(&window.device, &window.surface_desc);

					window.canvas.inner.size = Vec2::new(size.width as f32, size.height as f32);
				}
			}
			WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
				window.hidpi_factor = *scale_factor;
				let font_size = (13.0 * window.hidpi_factor) as f32;
				imgui.context.fonts().clear();
				imgui
					.context
					.fonts()
					.add_font(&[FontSource::DefaultFontData {
						config: Some(imgui::FontConfig {
							oversample_h: 1,
							pixel_snap_h: true,
							size_pixels: font_size,
							..Default::default()
						}),
					}]);
				imgui.renderer.reload_font_texture(
					&mut imgui.context,
					&window.device,
					&window.queue,
				);
			}
			WindowEvent::CloseRequested => event_loop.exit(),
			WindowEvent::RedrawRequested => {
				let now = Instant::now();
				imgui
					.context
					.io_mut()
					.update_delta_time(now - imgui.last_frame);
				imgui.last_frame = now;

				let frame = match window.surface.get_current_texture() {
					wgpu::CurrentSurfaceTexture::Success(frame) => frame,
					// Suboptimal is fine to render with — likely an
					// upcoming resize will reconfigure the surface.
					wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
					wgpu::CurrentSurfaceTexture::Timeout
					| wgpu::CurrentSurfaceTexture::Occluded => return,
					wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
						window
							.surface
							.configure(&window.device, &window.surface_desc);
						return;
					}
					other => {
						eprintln!("get_current_texture error: {other:?}");
						return;
					}
				};
				imgui
					.platform
					.prepare_frame(imgui.context.io_mut(), &window.window)
					.expect("Failed to prepare frame");

				let mut encoder: wgpu::CommandEncoder = window
					.device
					.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

				let view = frame
					.texture
					.create_view(&wgpu::TextureViewDescriptor::default());
				let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: None,
					color_attachments: &[Some(wgpu::RenderPassColorAttachment {
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(imgui.clear_color),
							store: wgpu::StoreOp::Store,
						},
						depth_slice: None,
					})],
					depth_stencil_attachment: None,
					timestamp_writes: None,
					occlusion_query_set: None,
					multiview_mask: None,
				});

				window.canvas.inner.draw_grid(&mut rpass);

				let ui = imgui.context.frame();

				{
					let imw = ui.window("w")
						.position([0.0, 0.0], Condition::Always)
						.size([ window.surface_desc.width as f32, window.surface_desc.height as f32 ], Condition::Always)
						.flags(WindowFlags::NO_BACKGROUND | WindowFlags::NO_MOVE | WindowFlags::NO_DECORATION |
							WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE)
						.menu_bar(false);

					imw.build(|| {
						window.canvas.draw(ui, &window.device, &window.surface_desc, &mut rpass, &mut window.queue);
					});
				}

				if imgui.last_cursor != ui.mouse_cursor() {
					imgui.last_cursor = ui.mouse_cursor();
					imgui.platform.prepare_render(ui, &window.window);
				}

				imgui
					.renderer
					.render(
						imgui.context.render(),
						&window.queue,
						&window.device,
						&mut rpass,
					)
					.expect("Rendering failed");

				drop(rpass);

				window.queue.submit(Some(encoder.finish()));

				frame.present();
			}
			_ => (),
		}

		imgui.platform.handle_event::<()>(
			imgui.context.io_mut(),
			&window.window,
			&Event::WindowEvent { window_id, event },
		);
	}

	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ()) {
		let window = self.window.as_mut().unwrap();
		let imgui = window.imgui.as_mut().unwrap();
		imgui.platform.handle_event::<()>(
			imgui.context.io_mut(),
			&window.window,
			&Event::UserEvent(event),
		);
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		device_id: winit::event::DeviceId,
		event: winit::event::DeviceEvent,
	) {
		let window = self.window.as_mut().unwrap();
		let imgui = window.imgui.as_mut().unwrap();
		imgui.platform.handle_event::<()>(
			imgui.context.io_mut(),
			&window.window,
			&Event::DeviceEvent { device_id, event },
		);
	}

	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		let window = self.window.as_mut().unwrap();
		let imgui = window.imgui.as_mut().unwrap();
		window.window.request_redraw();
		imgui.platform.handle_event::<()>(
			imgui.context.io_mut(),
			&window.window,
			&Event::AboutToWait,
		);
	}
}

fn main() {
	env_logger::init();

	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);
	event_loop.run_app(&mut App::default()).unwrap();
}
