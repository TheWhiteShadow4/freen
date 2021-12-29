
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text, Layout, GlyphBrush};
use wgpu::{Device, TextureFormat, CommandEncoder, TextureView};
use futures::task::SpawnExt;

use super::{Size, Buffer};

pub struct TextGrid
{
	glyph_brush: GlyphBrush<()>,
	screen_size: Size,
	font_size: u32,
	staging_belt: wgpu::util::StagingBelt,
	local_pool: futures::executor::LocalPool,
	local_spawner: futures::executor::LocalSpawner
}

impl TextGrid
{
	pub fn new(device: &Device, screen_size: Size, font_size: u32, texture_format: TextureFormat) -> Self
	{
		let font = ab_glyph::FontArc::try_from_slice(include_bytes!("fira-mono.regular.ttf")).unwrap();
		let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, texture_format);

		let staging_belt = wgpu::util::StagingBelt::new(1024);
		let local_pool = futures::executor::LocalPool::new();
		let local_spawner = local_pool.spawner();

		Self
		{
			glyph_brush,
			screen_size,
			font_size,
			staging_belt,
			local_pool,
			local_spawner,
		}
	}

	pub fn draw(&mut self, device: &Device, encoder: &mut CommandEncoder, view: &TextureView, buffer: &Buffer)
	{
		let bounds = (self.screen_size.width as f32, self.screen_size.height as f32);
		let cell_size = Size{width: self.font_size / 2, height: self.font_size};

		for i in 0..buffer.chars.len()
		{
			if buffer.chars[i].is_whitespace() { continue; }
			let x = ((i as u32 % buffer.width) * cell_size.width) as f32;
			let y = ((i as u32 / buffer.width) * cell_size.height) as f32;
			self.glyph_brush.queue(Section {
				screen_position: (x, y),
				bounds: bounds,
				text: vec![Text::new(&buffer.chars[i].to_string()).with_color(buffer.foreground[i]).with_scale(self.font_size as f32)],
				layout: Layout::default()
			});
		}



		self.glyph_brush
		.draw_queued(
			&device,
			&mut self.staging_belt,
			encoder,
			view,
			self.screen_size.width,
			self.screen_size.height,
		)
		.expect("Draw queued");
		self.staging_belt.finish();
	}

	pub fn clean_frame(&mut self)
	{
		self.local_spawner.spawn(self.staging_belt.recall()).expect("Recall staging belt");
		self.local_pool.run_until_stalled();
	}
}