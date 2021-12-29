
use crate::{EventEmitter, component::{UID, Component, generateUID}, Event};
use super::{Buffer, Color, Size, ScreenSize};
use super::renderer::Renderer;

use std::{sync::{Arc, Mutex}, thread, fmt::Debug};
use winit::platform::windows::EventLoopExtWindows;
use winit::window::Icon;
use winit::{
	event,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}, dpi::PhysicalSize,
	platform::run_return::EventLoopExtRunReturn
};
use winit::event::{WindowEvent, ElementState, MouseButton};

const EVENT_MOUSE_DOWN: &str = "OnMouseDown\0";
const EVENT_MOUSE_UP: &str = "OnMouseUp\0";
const EVENT_MOUSE_MOVE: &str = "OnMouseMove\0";


#[derive(Debug)]
pub struct ScreenComponent
{
	id: UID,
	pub fg: Color,
	pub bg: Color,
	font_size: u32,
	buffer: Arc<Mutex<Buffer>>,
	window: Arc<Mutex<Option<Window>>>,
}

impl ScreenComponent
{
	pub fn new(width: u32, height: u32, font_size: u32) -> Self
	{
		assert!(width > 0);
		assert!(height > 0);
		env_logger::init();
		Self
		{
			id: generateUID(),
			fg: Color::WHITE,
			bg: Color::BLACK,
			font_size,
			buffer: Arc::new( Mutex::new(Buffer::new(width, height))),
			window: Arc::new( Mutex::new(None))
		}
	}

	pub fn open(&self, emitter: Option<EventEmitter>)
	{
		let buf = self.buffer.lock().unwrap();
		let width = buf.width;
		let height = buf.height;
		let font_size = self.font_size;
		let buffer = self.buffer.clone();
		let window_arc = self.window.clone();
		thread::spawn(move || {
			let (mut screen, event_loop) = Screen::new(width, height, font_size, emitter, buffer, window_arc);
			screen.run_event_loop(event_loop);
		});
	}

	pub fn flush(&self)
	{
		let window = self.window.lock().unwrap();
		if window.is_some()
		{
			window.as_ref().unwrap().request_redraw();
		}
	}

	pub fn withBuffer<F>(&self, mut func: F) where F: FnMut(&mut Buffer)
	{
		func(&mut self.buffer.lock().unwrap());
	}

	pub fn isOpen(&self) -> bool
	{
		self.window.lock().unwrap().is_some()
	}

	/*pub fn set_emitter(&mut self, emitter: Option<EventEmitter>)
	{
		self.share.lock().unwrap().setEmitter(emitter);
	}*/

	pub fn resize_screen(&mut self, width: u32, height: u32)
	{
		self.buffer.lock().unwrap().resize(width, height);
	}
}

struct Screen
{
	size: ScreenSize,
	input: InputHelper,
	buffer: Arc<Mutex<Buffer>>,
	window: Arc<Mutex<Option<Window>>>,
	renderer: Renderer
}

impl Debug for Screen
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
        f.debug_struct("Screen").field("size", &self.size)
		.field("buffer", &self.buffer)
		.field("window", &self.window).finish()
    }
}

impl Screen
{
	fn new(width: u32, height: u32, font_size: u32, emitter: Option<EventEmitter>, buffer: Arc<Mutex<Buffer>>, window_arc: Arc<Mutex<Option<Window>>>) -> (Self, EventLoop<()>)
	{
		let input = InputHelper::new(emitter, width, height);

		let size = ScreenSize::from_grid(width, height, font_size);

		let iconData = include_bytes!("icon.rgba");
		let icon = Icon::from_rgba(iconData.to_vec(), 32, 32).ok();

		let event_loop: EventLoop<()>  = EventLoop::new_any_thread();

		let window = WindowBuilder::new()
		.with_title("Screen")
		.with_inner_size(PhysicalSize{width: size.window_width as f32, height: size.window_height as f32})
		.with_maximized(false)
		.with_resizable(false)
		.with_window_icon(icon)
		.build(&event_loop)
		.unwrap();

		let renderer = Renderer::new( &window, size, wgpu::PresentMode::Mailbox);
		window_arc.lock().unwrap().replace(window);

		(Self
		{
			size,
			input,
			buffer,
			window: window_arc,
			renderer
		}, event_loop)
	}

	fn run_event_loop(&mut self, mut event_loop: EventLoop<()>)
	{
		//let buffer = self.buffer;

		event_loop.run_return(|event, _, control_flow| {

			//share.lock().unwrap().update(&mut renderer);
	
			if let event::Event::RedrawRequested(_) = event
			{
				self.perfom_resizeing();
				if self.renderer.render(&self.buffer.lock().unwrap())
				{
					*control_flow = ControlFlow::Exit;
					//title = "Screen ".to_string() + &fps_counter.tick().to_string();
					//share.
					//window.set_inner_size(PhysicalSize{width: 500.0, height: 400.0});
					//window.set_title(&title);
					return;
				}
				//println!("FPS: {}", fps_counter.tick());
			}
	
			self.input.update(&event);
			if self.input.closeRequested()
			{
				*control_flow = ControlFlow::Exit;
				return;
			}
		});
		self.window.lock().unwrap().take();
	}

	fn perfom_resizeing(&mut self)
	{
		let buffer = self.buffer.lock().unwrap();
		if buffer.width != self.size.grid_width || buffer.height != self.size.grid_height
		{
			self.size.resize_grid(buffer.width, buffer.height);
			self.window.lock().unwrap().as_ref().unwrap().set_inner_size(PhysicalSize{width: self.size.window_width as f32, height: self.size.window_height as f32});
			self.renderer.resize(self.size);
		}
	}
}

impl Component for ScreenComponent
{
	fn uid(&self) -> UID { self.id }

	fn listen(&mut self, _emitter: Option<EventEmitter>)
	{
		todo!()
    }
}

struct InputHelper
{
	emitter: Option<EventEmitter>,
	grid: Size,
	mouseX: i32,
	mouseY: i32,
	close: bool
}

impl InputHelper
{
	pub fn new(emitter: Option<EventEmitter>, width: u32, height: u32) -> Self
	{
		Self{emitter, grid: Size{width, height}, mouseX: 0, mouseY: 0, close: false}
	}

	pub fn update<T>(&mut self, event: &event::Event<T>)
	{
		if self.emitter.is_some()
		{
			match &event
			{
				event::Event::WindowEvent { event, .. } => self.handleWindowEvents(event),
				_ => {}
			}
		}
	}

	fn handleWindowEvents(&mut self, event: &WindowEvent)
	{
		let em = self.emitter.as_mut().unwrap();
		match event {
			WindowEvent::MouseInput {
				state: ElementState::Pressed,
				button,
				..
			} => {
				em.send(Event
					{
						eventType: EVENT_MOUSE_DOWN.to_string(),
						component: em.owner(),
						arg1: self.mouseX,
						arg2: self.mouseY,
						arg3: mouse_button_to_int(button)});
			},
			WindowEvent::MouseInput {
				state: ElementState::Released,
				button,
				..
			} => {
				em.send(Event
					{
						eventType: EVENT_MOUSE_UP.to_string(),
						component: em.owner(),
						arg1: self.mouseX,
						arg2: self.mouseY,
						arg3: mouse_button_to_int(button)});
			},
			WindowEvent::CursorMoved {
				position,
				..
			} => {
				self.mouseX = (position.x / self.grid.width as f64) as i32;
				self.mouseY = (position.y / self.grid.height as f64) as i32;
				em.send(Event
					{
						eventType: EVENT_MOUSE_MOVE.to_string(),
						component: em.owner(),
						arg1: self.mouseX,
						arg2: self.mouseY,
						arg3: 0});
			},

			WindowEvent::CloseRequested => self.close = true,
			_ => {}
		}
	}

	fn closeRequested(&self) -> bool
	{
		self.close
	}
}

fn mouse_button_to_int(button: &MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Other(byte) => *byte as i32,
    }
}