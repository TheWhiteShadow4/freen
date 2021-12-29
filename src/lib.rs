#![allow(non_snake_case)]

mod component;
use crate::component::*;

mod screens;
use crate::screens::*;
use screens::renderer::Renderer;

use core::time;
use std::collections::VecDeque;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::{thread, ptr};
use std::sync::{Arc, Mutex};

use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::platform::windows::EventLoopExtWindows;
use winit::window::Icon;
use winit::{
	event,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window}, dpi::PhysicalSize,
	platform::run_return::EventLoopExtRunReturn
};


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
		env_logger::init();
		Screen
		{
			id: component::generateUUID(),
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
		screenShare.window().unwrap().request_redraw();
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

	fn setEmitter(&mut self, emitter: Option<EventEmitter>)
	{
		self.share.lock().unwrap().setEmitter(emitter);
	}

	fn resize_screen(&mut self, width: u32, height: u32)
	{
		let share = &mut self.share.lock().unwrap();
		share.buffer.resize(width, height);
	}
}

impl Component for Screen
{
	fn uid(&self) -> UID { self.id }
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

fn mouse_button_to_int(button: &MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Other(byte) => *byte as i32,
    }
}

const EVENT_MOUSE_DOWN: &str = "OnMouseDown\0";
const EVENT_MOUSE_UP: &str = "OnMouseUp\0";
const EVENT_MOUSE_MOVE: &str = "OnMouseMove\0";

#[derive(Debug)]
pub struct Event
{
	pub eventType: String,
	pub component: UID,
	pub arg1: i32,
	pub arg2: i32,
	pub arg3: i32
}

#[derive(Debug)]
pub struct EventEmitter
{
	queue: Arc<Mutex<VecDeque<Event>>>,
	owner: UID
}

impl EventEmitter
{
	fn send(&mut self, event: Event)
	{
		//println!("{:?}", event);
		self.queue.lock().unwrap().push_back(event);
	}

	fn owner(&self) -> UID { self.owner }
}

#[derive(Debug)]
pub struct EventHandler
{
	eventQueue: Arc<Mutex<VecDeque<Event>>>,
}

impl EventHandler
{
	fn new(size: usize) -> Self
	{
		Self
		{
			eventQueue: Arc::new( Mutex::new(VecDeque::<Event>::with_capacity(size)))
		}
	}

	fn newEmitter(&self, owner: UID) -> EventEmitter
	{
		EventEmitter{queue: self.eventQueue.clone(), owner}
	}

	fn next(&mut self) -> Option<Event>
	{
		self.eventQueue.lock().unwrap().pop_front()
	}
}

#[repr(C)]
pub struct ExtEvent
{
	pub eventType: *const u8,
	pub component: u32,
	pub arg1: i32,
	pub arg2: i32,
	pub arg3: i32
}

impl ExtEvent
{
	pub fn empty() -> Self
	{
		Self
		{
			eventType: ptr::null(),
			component: 0,//EMPTY_UID,
			arg1: 0,
			arg2: 0,
			arg3: 0
		}
	}
}

//const event: Event = Event{eventType: 0, component: 0};

#[no_mangle]
pub unsafe extern "C" fn newEventHandler() -> *const EventHandler
{
	let handler = EventHandler::new(32);
	let boxed = Box::new(handler);
    Box::into_raw(boxed)
}

#[no_mangle]
pub unsafe extern "C" fn startListen(hptr: *mut EventHandler, sptr: *mut Screen)
{
	let emitter = (*hptr).newEmitter((*sptr).uid());
	(*sptr).setEmitter(Some(emitter));
}

#[no_mangle]
pub unsafe extern "C" fn pull(ptr: *mut EventHandler, t: f32) -> ExtEvent
{
	match (*ptr).next()
	{
		Some(evt) =>
		{
			return ExtEvent{
				eventType: evt.eventType.as_ptr(),
				component: 1,//evt.component,
				arg1: evt.arg1,
				arg2: evt.arg2,
				arg3: evt.arg3,
			};
		}
		None => if t > 0.0
		{
			let millis = time::Duration::from_millis((t * 1000.0) as u64);
			thread::sleep(millis);
		}
	}
	//let eventPtr = Box::into_raw(Box::new(newEvent));
	//handler.event = eventPtr;
	//newEvent
	ExtEvent::empty()
	//ptr::null()
}

#[no_mangle]
pub extern "C" fn create(width: u32, height: u32, fontsize: u32, handler: *mut EventHandler) -> *mut Screen
{
	let screen = Screen::new(width, height, fontsize);
	unsafe
	{
		if handler.is_null()
		{
			screen.open(None);
		}
		else
		{
			screen.open(Some((*handler).newEmitter(screen.uid())));
		}
	}
	let boxed = Box::new(screen);
    Box::into_raw(boxed)
}

/*
#[no_mangle]
pub unsafe extern "C" fn open(sptr: *mut Screen, hptr: *mut EventHandler)
{
	if hptr.is_null()
	{
		(*sptr).open(None);
	}
	else
	{
		(*sptr).open(Some((*hptr).newEmitter((*sptr).uid())));
	}
}*/

#[no_mangle]
pub unsafe extern "C" fn destroy(ptr: *mut Screen)
{
	if !ptr.is_null() { Box::from_raw(ptr); }
}

#[no_mangle]
pub unsafe extern "C" fn foreground(ptr: *mut Screen, r: f32, g: f32, b: f32, a: f32)
{
	(*ptr).fg = Color::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn background(ptr: *mut Screen, r: f32, g: f32, b: f32, a: f32)
{
	(*ptr).bg = Color::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn fill(ptr: *const Screen, x: i32, y: i32, w: i32, h: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = &*ptr;

	handle.withBuffer(|buffer| {
		buffer.fill(x, y, w, h, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn writeText(ptr: *mut Screen, x: i32, y: i32, cstr: *const c_char)
{
	let text = CStr::from_ptr(cstr).to_str().expect("Ungültige Zeichen");
	let handle = &*ptr;
	handle.withBuffer(|buffer| {
		buffer.writeText(x, y, &text, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn write(ptr: *mut Screen, x: i32, y: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = &*ptr;
	handle.withBuffer(|buffer| {
		buffer.write(x, y, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn setSize(ptr: *mut Screen, width: u32, height: u32)
{
	assert!(width > 0);
	assert!(height > 0);

	(*ptr).resize_screen(width, height);
}

#[no_mangle]
pub unsafe extern "C" fn flush(ptr: *mut Screen)
{
	(*ptr).flush();
}

unsafe fn CStr2Char(cstr: *const c_char) -> char
{
	let str = CStr::from_ptr(cstr).to_str().expect("Ungültiges Zeichen");
	str.chars().nth(0).expect("Ungültiges Zeichen")
}
