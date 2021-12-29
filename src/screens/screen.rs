
use crate::{EventEmitter, component::{UID, Component, generateUID}, Event};
use super::{Buffer, Color, Size};
use super::renderer::Renderer;

use std::{sync::{Arc, Mutex}, thread};
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
pub struct ScreenShare
{
	window: Option<Window>,
	buffer: Buffer,
	width: u32,
	height: u32,
	fontsize: u32,
	emitter: Option<EventEmitter>
}

impl ScreenShare
{
	pub fn new(width: u32, height: u32, fontsize: u32) -> ScreenShare
	{
		ScreenShare{
			window: None,
			buffer: Buffer::new(width, height),
			width,
			height,
			fontsize,
			emitter: None
		}
	}

	pub fn window(&self) -> Option<&Window>
	{
		self.window.as_ref()
	}

	fn setEmitter(&mut self, emitter: Option<EventEmitter>)
	{
		self.emitter = emitter;
	}
}

#[derive(Debug)]
pub struct ScreenComponent
{

}

impl ScreenComponent
{

}

#[derive(Debug)]
pub struct Screen
{
	id: UID,
	pub share: Arc<Mutex<ScreenShare>>,
	pub fg: Color,
	pub bg: Color
}

impl Screen
{
	pub fn new(width: u32, height: u32, fontsize: u32) -> Screen
	{
		assert!(width > 0);
		assert!(height > 0);
		env_logger::init();
		Screen
		{
			id: generateUID(),
			share: Arc::new( Mutex::new(ScreenShare::new(width, height, fontsize))),
			fg: Color::WHITE,
			bg: Color::BLACK
		}
	}

	pub fn open(&self, emitter: Option<EventEmitter>)
	{
		let share = self.share.clone();
		thread::spawn(|| {
			openWindow(share, emitter);
		});
	}

	pub fn flush(&self)
	{
		let screenShare = self.share.lock().unwrap();
		match screenShare.window()
		{
			Some(w) => w.request_redraw(),
			None => ()
		}
	}

	pub fn withBuffer<F>(&self, mut func: F) where F: FnMut(&mut Buffer)
	{
		let buffer = &mut self.share.lock().unwrap().buffer;
		func(buffer);
	}

	pub fn isOpen(&self) -> bool
	{
		self.share.lock().unwrap().window().is_some()
	}

	pub fn set_emitter(&mut self, emitter: Option<EventEmitter>)
	{
		self.share.lock().unwrap().setEmitter(emitter);
	}

	pub fn resize_screen(&mut self, width: u32, height: u32)
	{
		let share = &mut self.share.lock().unwrap();
		share.buffer.resize(width, height);
	}
}

impl Component for Screen
{
	fn uid(&self) -> UID { self.id }
}

fn openWindow(share: Arc<Mutex<ScreenShare>>, emitter: Option<EventEmitter>)
{
	let mut cnf = share.lock().unwrap();
	let font_size = cnf.fontsize;

	let grid_size = Size{width: cnf.width, height: cnf.height};
	let screen_size = Size{width: (cnf.width * font_size)/2, height: cnf.height * font_size};

	let mut inputHelper = InputHelper::new(emitter, grid_size.width, grid_size.height);
	
	let iconData = include_bytes!("icon.rgba");
	let icon = Icon::from_rgba(iconData.to_vec(), 32, 32).ok();

	let mut event_loop: EventLoop<()>  = EventLoop::new_any_thread();
	let window = WindowBuilder::new()
	.with_title("Screen")
	.with_inner_size(PhysicalSize{width: screen_size.width as f32, height: screen_size.height as f32})
	.with_maximized(false)
	.with_resizable(false)
	.with_window_icon(icon)
	.build(&event_loop)
	.unwrap();

	let mut renderer = Renderer::new( &window, grid_size, font_size, wgpu::PresentMode::Mailbox);
	cnf.window = Some(window);
	drop(cnf);

	//let mut fps_counter = FPSCounter::new();

	event_loop.run_return(|event, _, control_flow| {

		//share.lock().unwrap().update(&mut renderer);

		if let event::Event::RedrawRequested(_) = event
		{
			if renderer.render(&mut share.lock().unwrap().buffer)
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

		inputHelper.update(&event);

		if inputHelper.closeRequested()
		{
			*control_flow = ControlFlow::Exit;
			return;
		}
	});
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