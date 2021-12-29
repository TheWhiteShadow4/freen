#![allow(non_snake_case)]
#![allow(unused_parens)]

use winit::window::Window;

use super::ScreenSize;
use super::{grid::PixelGrid, Buffer};

use super::text::TextGrid;

pub struct Renderer
{
	device: wgpu::Device,
	queue: wgpu::Queue,
	surface: wgpu::Surface,
	present_mode: wgpu::PresentMode,
	grid: PixelGrid,
	text: TextGrid,
}

impl Renderer
{
	pub fn new(window: &Window, size: ScreenSize, present_mode: wgpu::PresentMode) -> Self
	{
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(&window) };

		// Initialize GPU
		let (device, queue) = futures::executor::block_on(async {
			let adapter = instance
				.request_adapter(&wgpu::RequestAdapterOptions {
					power_preference: wgpu::PowerPreference::LowPower,
					compatible_surface: Some(&surface),
					force_fallback_adapter: false,
				})
				.await
				.expect("Request adapter");

			adapter
				.request_device(&wgpu::DeviceDescriptor::default(), None)
				.await
				.expect("Request device")
		});

		let texture_format = wgpu::TextureFormat::Bgra8UnormSrgb;
		let grid = PixelGrid::new(&device,
			size,
			texture_format);

		surface.configure(
			&device,
			&wgpu::SurfaceConfiguration {
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
				format: texture_format,
				width: size.window_width,
				height: size.window_height,
				present_mode,
			},
		);

		let text = TextGrid::new(&device, size, texture_format);

		Self{
			device,
			queue,
			surface,
			present_mode,
			grid,
			text
		}
	}

	pub fn resize(&mut self, size: ScreenSize)
	{
		self.surface.configure(
			&self.device,
			&wgpu::SurfaceConfiguration {
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
				format: wgpu::TextureFormat::Bgra8UnormSrgb,
				width: size.window_width,
				height: size.window_height,
				present_mode: self.present_mode,
			},
		);

		self.grid.resize(&self.device, size, &self.queue);
		self.text.resize(&self.device, size);
	}

	pub fn render(&mut self, buffer: &Buffer) -> bool
	{
		let mut encoder = self.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {
				label: Some("Redraw"),
			},
		);

		// Get the next frame
		let frame = self.surface.get_current_texture().expect("Get next frame");
		let view = &frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		// Clear frame
		encoder.begin_render_pass(
			&wgpu::RenderPassDescriptor {
				label: Some("Render pass"),
				color_attachments: &[
					wgpu::RenderPassColorAttachment {
						view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(
								wgpu::Color {
									r: 0.5,
									g: 0.4,
									b: 0.0,
									a: 1.0,
								},
							),
							store: true,
						},
					},
				],
				depth_stencil_attachment: None,
			},
		);

		// Zeichne den Hintergrund
		self.grid.draw_queued(&self.device, &self.queue, &mut encoder, view, buffer);
		// Zeichen den Vordergrund
		self.text.draw(&self.device, &mut encoder, view, buffer);

		self.queue.submit(Some(encoder.finish()));
		frame.present();
		
		self.text.clean_frame();

		false
	}
}
