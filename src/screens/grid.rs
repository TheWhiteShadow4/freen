use std::num::NonZeroU32;

use wgpu::{Device, RenderPipeline, Extent3d};
use wgpu::util::DeviceExt;
use ultraviolet::Mat4;

use super::{Size, Buffer};


pub struct PixelGrid
{
	renderer: GridRenderer,
	texture_format: wgpu::TextureFormat,
	screen_size: Size
}

impl PixelGrid
{
	pub fn new(
		device: &Device,
		grid_size: Size,
		screen_size: Size,
		texture_format: wgpu::TextureFormat) -> Self
	{
		let texture_extent = wgpu::Extent3d {
			width: grid_size.width,
			height: grid_size.height,
			depth_or_array_layers: 1,
		};

		let texture = PixelGrid::create_texture(&device, grid_size);

		let renderer = GridRenderer::new(&device, texture, texture_format, texture_extent, screen_size);

		/*buffer[0] = 0xff;
		buffer[1] = 0xff;
		buffer[2] = 0xff;
		buffer[3] = 0xff;
		buffer[5] = 0xff;
		buffer[7] = 0xff;

		buffer[26] = 0xff;
		buffer[27] = 0xff;

		buffer[54] = 0xff;
		buffer[55] = 0xff;

		buffer[130] = 0xff;
		buffer[131] = 0xff;*/

		Self
		{
			renderer,
			texture_format,
			screen_size
		}
	}

	pub fn draw_queued(&self, device: &Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, buffer: &Buffer)
	{
		self.renderer.draw(device, queue, encoder, buffer, target);
	}

	pub fn resize_grid(&mut self, device: &Device, width: u32, height: u32)
	{
		let texture = PixelGrid::create_texture(device, Size{width, height});
		let texture_extent = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};

		self.renderer = GridRenderer::new(&device, texture, self.texture_format, texture_extent, self.screen_size);
	}

	fn create_texture(device: &Device, grid_size: Size) -> wgpu::Texture
	{
		let texture_extent = wgpu::Extent3d {
			width: grid_size.width,
			height: grid_size.height,
			depth_or_array_layers: 1,
		};

		//let buffer_size = (grid_size.width * grid_size.height * 4) as usize;

		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("grid:source_texture"),
			size: texture_extent,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});
		texture
	}
}

struct GridRenderer
{
	pipeline: RenderPipeline,
	bind_group: wgpu::BindGroup,
	vertex_buffer: wgpu::Buffer,
	clip_rect: (u32, u32, u32, u32),
	texture: wgpu::Texture,
	texture_size: Extent3d,
}

impl GridRenderer
{
	fn new(
		device: &Device,
		texture: wgpu::Texture,
		texture_format: wgpu::TextureFormat,
		texture_size: Extent3d,
		screen_size: Size) -> GridRenderer
	{
		let shader = device.create_shader_module(&wgpu::include_wgsl!("../shaders/shader.wgsl"));

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("grid:renderer_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
			lod_max_clamp: 1.0,
			..Default::default()
        });

		let vertex_data: [[f32; 2]; 3] = [
            [-1.0, -1.0],
            [3.0, -1.0],
            [-1.0, 3.0],
        ];
		let vertex_data_slice: &[u8] = bytemuck::cast_slice(&vertex_data);
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("grid:renderer_vertex_buffer"),
			contents: vertex_data_slice,
			usage: wgpu::BufferUsages::VERTEX,
		});

		let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (vertex_data_slice.len() / vertex_data.len()) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        };
		
		let matrix = ScalingMatrix::new(
            (texture_size.width as f32, texture_size.height as f32),
            (screen_size.width as f32, screen_size.height as f32),
        );
		let transform_bytes = matrix.as_bytes();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid:renderer_matrix_uniform_buffer"),
            contents: transform_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("grid:renderer_bind_group_layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 2,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		});
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("grid:renderer_bind_group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&sampler),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: uniform_buffer.as_entire_binding(),
				},
			],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("grid:renderer_pipeline_layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("grid:renderer_pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[vertex_buffer_layout],
			},
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[wgpu::ColorTargetState {
					format: texture_format,
					blend: Some(wgpu::BlendState {
						color: wgpu::BlendComponent::REPLACE,
						alpha: wgpu::BlendComponent::REPLACE,
					}),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			multiview: None,
		});

		let clip_rect = matrix.clip_rect();

		GridRenderer
		{
			pipeline,
			bind_group,
			vertex_buffer,
			clip_rect,
			texture,
			texture_size
		}
	}

	fn draw(&self,
		_device: &wgpu::Device,
		queue: &wgpu::Queue,
		encoder: &mut wgpu::CommandEncoder,
		buffer: &Buffer,
		target: &wgpu::TextureView)
	{
		let bytes_per_row = (self.texture_size.width as f32 * 4.0) as u32;
		let buffer_data: Vec<u8> = buffer.background.iter().flat_map(|col| col.bytes().to_vec().into_iter()).collect();
		queue.write_texture(
			wgpu::ImageCopyTexture {
				texture: &self.texture,
				mip_level: 0,
				origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
				aspect: wgpu::TextureAspect::All,
			},
			&buffer_data,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: NonZeroU32::new(bytes_per_row),
				rows_per_image: NonZeroU32::new(self.texture_size.height),
			},
			self.texture_size,
		);

		let mut rpass =
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("grid::pipeline render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

		rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_scissor_rect(
            self.clip_rect.0,
            self.clip_rect.1,
            self.clip_rect.2,
            self.clip_rect.3,
        );
        rpass.draw(0..3, 0..1);
	}
}

#[derive(Debug)]
pub(crate) struct ScalingMatrix {
    pub(crate) transform: Mat4,
    clip_rect: (u32, u32, u32, u32),
}

impl ScalingMatrix {
    // texture_size is the dimensions of the drawing texture
    // screen_size is the dimensions of the surface being drawn to
    pub(crate) fn new(texture_size: (f32, f32), screen_size: (f32, f32)) -> Self {
        let (texture_width, texture_height) = texture_size;
        let (screen_width, screen_height) = screen_size;

        // Get smallest scale size
        let hscale = (screen_width / texture_width).max(1.0).floor();
		let vscale = (screen_height / texture_height).max(1.0).floor();

        let scaled_width = texture_width * hscale;
        let scaled_height = texture_height * vscale;

        // Create a transformation matrix
        let sw = scaled_width / screen_width;
        let sh = scaled_height / screen_height;
        let tx = (texture_width / screen_width - 1.0).max(0.0);
        let ty = (1.0 - texture_height / screen_height).min(0.0);
		//let ty = (texture_height / screen_height - 1.0).max(0.0);
        #[rustfmt::skip]
        let transform: [f32; 16] = [
            sw,  0.0, 0.0, 0.0,
            0.0, sh, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx,  ty, 0.0, 1.0,
        ];

        // Create a clipping rectangle
        let clip_rect = {
            let scaled_width = scaled_width.min(screen_width);
            let scaled_height = scaled_height.min(screen_height);
            let x = ((screen_width - scaled_width) / 2.0) as u32;
            let y = ((screen_height - scaled_height) / 2.0) as u32;

            (x, y, scaled_width as u32, scaled_height as u32)
        };

        Self {
            transform: Mat4::from(transform),
            clip_rect,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        self.transform.as_byte_slice()
    }

    pub(crate) fn clip_rect(&self) -> (u32, u32, u32, u32) {
        self.clip_rect
    }
}