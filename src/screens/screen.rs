
use crate::*;
use super::{Buffer, Color, ScreenSize};
use super::renderer::Renderer;

use std::{sync::{Arc, Mutex}, thread, fmt::Debug};
use fps_counter::FPSCounter;
use winit::{platform::windows::EventLoopExtWindows, dpi::PhysicalPosition};
use winit::window::Icon;
use winit::{
	event,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}, dpi::PhysicalSize,
	platform::run_return::EventLoopExtRunReturn
};
use winit::event::{WindowEvent, ElementState, MouseButton};


const EVENT_WINDOW_CLOSED: &str = "WindowClosed\0";
const EVENT_MOUSE_DOWN: &str = "OnMouseDown\0";
const EVENT_MOUSE_UP: &str = "OnMouseUp\0";
const EVENT_MOUSE_MOVE: &str = "OnMouseMove\0";
const EVENT_KEY_DOWN: &str = "OnKeyDown\0";
const EVENT_KEY_UP: &str = "OnKeyUp\0";

pub struct ScreenComponent
{
	id: UID,
	pub fg: Color,
	pub bg: Color,
	font_size: u32,
	window: Arc<Mutex<Option<Window>>>,
	emitter: Option<EventEmitter>
}

impl ScreenComponent
{
	pub fn new(font_size: u32, ) -> Self
	{
		Self
		{
			id: generateUID(),
			fg: Color::WHITE,
			bg: Color::BLACK,
			font_size,
			window: Arc::new( Mutex::new(None)),
			emitter: None
		}
	}

	pub fn set_emitter(&mut self, emitter: Option<EventEmitter>)
	{
		self.emitter = emitter;
	}

	pub fn open(&mut self, buffer: Arc<Mutex<Buffer>>)
	{
		let font_size = self.font_size;
		let emitter = self.emitter.take();
		let buffer_arc = buffer.clone();
		let window_arc = self.window.clone();
		thread::spawn(move || {
			let (mut screen, event_loop) = Screen::new(font_size, emitter, buffer_arc, window_arc);
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

	pub fn isOpen(&self) -> bool
	{
		self.window.lock().unwrap().is_some()
	}

	pub fn screen_location(&mut self, x: i32, y: i32)
	{
		match self.window.lock().unwrap().as_ref()
		{
			Some(w) => w.set_outer_position(PhysicalPosition{x, y}),
			None => {}
		}
	}
}

struct Screen
{
	size: ScreenSize,
	input: InputHelper,
	buffer: Arc<Mutex<Buffer>>,
	window: Arc<Mutex<Option<Window>>>,
	title: String,
	renderer: Renderer,
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
	fn new(font_size: u32, emitter: Option<EventEmitter>, buffer: Arc<Mutex<Buffer>>, window_arc: Arc<Mutex<Option<Window>>>) -> (Self, EventLoop<()>)
	{
		let buf = buffer.lock().unwrap();
		let width = buf.width;
		let height = buf.height;
		drop(buf);
		let input = InputHelper::new(emitter, width, height);

		let size = ScreenSize::from_grid(width, height, font_size);

		let iconData = include_bytes!("icon.rgba");
		let icon = Icon::from_rgba(iconData.to_vec(), 32, 32).ok();

		let event_loop: EventLoop<()> = EventLoop::new_any_thread();

		let title = "Screen".to_string();
		let window = WindowBuilder::new()
		.with_title(&title)
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
			title,
			renderer
		}, event_loop)
	}

	fn run_event_loop(&mut self, mut event_loop: EventLoop<()>)
	{
		let mut fps_counter = FPSCounter::new();

		event_loop.run_return(|event, _, control_flow| {

			if let event::Event::RedrawRequested(_) = event
			{
				self.perfom_resizeing();
				if self.renderer.render(&self.buffer.lock().unwrap())
				{
					*control_flow = ControlFlow::Exit;
					return;
				}
				self.title = format!("Screen {} fps", &fps_counter.tick());
				self.window.lock().unwrap().as_mut().unwrap().set_title(&self.title);
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
			self.input.resize(buffer.width, buffer.height);
		}
	}
}

impl Component for ScreenComponent
{
	fn uid(&self) -> UID { self.id }

	fn listen(&mut self, emitter: Option<EventEmitter>)
	{
		self.emitter = emitter;
    }
}

struct InputHelper
{
	emitter: Option<EventEmitter>,
	width: u32,
	height: u32,
	mouseX: i32,
	mouseY: i32,
	modifiers: i32,
	close: bool
}

impl InputHelper
{
	pub fn new(emitter: Option<EventEmitter>, width: u32, height: u32) -> Self
	{
		Self{emitter, width, height, mouseX: 0, mouseY: 0, modifiers: 0, close: false }
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

	pub fn resize(&mut self, width: u32, height: u32)
	{
		self.width = width;
		self.height = height;
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
				em.send(Signal::numArgs(
					EVENT_MOUSE_DOWN,
					em.owner(), vec![
					self.mouseX,
					self.mouseY,
					mouse_button_to_int(button)
				]));
			},
			WindowEvent::MouseInput {
				state: ElementState::Released,
				button,
				..
			} => {
				em.send(Signal::numArgs(
					EVENT_MOUSE_UP,
					em.owner(), vec![
					self.mouseX,
					self.mouseY,
					mouse_button_to_int(button)
				]));
			},
			WindowEvent::CursorMoved {
				position,
				..
			} => {
				let mouseX = (position.x / self.width as f64) as i32;
				let mouseY = (position.y / self.height as f64) as i32;
				if mouseX == self.mouseX && mouseY == self.mouseY {return;}
				self.mouseX = mouseX;
				self.mouseY = mouseY;
				em.send(Signal::numArgs(
					EVENT_MOUSE_MOVE,
					em.owner(), vec![
					self.mouseX,
					self.mouseY
				]));
			},
			WindowEvent::KeyboardInput {
				input,
				..
			} => {
				let eventType = match input.state
				{
        			ElementState::Pressed => EVENT_KEY_DOWN,
        			ElementState::Released => EVENT_KEY_UP,
    			};
				let key = match input.virtual_keycode
				{
					Some(k) => k as i32,
					None => 0
				};
				em.send(Signal::numArgs(
					eventType,
					em.owner(), vec![
					input.scancode as i32,
					key, self.modifiers
				]));
			},
			WindowEvent::ModifiersChanged(modifiers) => {
				let mut bits = 0i32;
				//if mouseleft() { bits |=  1 << 0; }
				//if mouseright() { bits |=  1 << 1; }
				if modifiers.ctrl()  { bits |= 1 << 2; }
				if modifiers.shift() { bits |= 1 << 3; }
				if modifiers.alt()   { bits |= 1 << 4; }
				if modifiers.logo()  { bits |= 1 << 5; }
				self.modifiers = bits;
			},
			WindowEvent::CloseRequested => {
				self.close = true;
				em.send(Signal::noArgs(EVENT_WINDOW_CLOSED, em.owner()));
			},
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