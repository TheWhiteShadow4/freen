#![allow(non_snake_case)]

mod component;
use crate::component::*;

mod screens;
use crate::screens::*;
use screens::screen::*;

use core::time;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::mpsc;
use std::time::Duration;
use std::ptr;
use std::sync::{Arc, Mutex};


#[derive(Debug, Default)]
pub struct Event
{
	pub eventType: String,
	pub component: UID,
	pub arg1: i32,
	pub arg2: i32,
	pub arg3: i32
}

pub struct EventEmitter
{
	sender: Arc<Mutex<mpsc::Sender<Event>>>,
	owner: UID
}

impl EventEmitter
{
	fn send(&mut self, event: Event)
	{
		if let Err(e) = self.sender.lock().unwrap().send(event)
		{
			eprintln!("Event Error {}", e);
		}
	}

	fn owner(&self) -> UID { self.owner }
}

pub struct EventHandler
{
	sender: Arc<Mutex<mpsc::Sender<Event>>>,
	recever: Arc<Mutex<mpsc::Receiver<Event>>>,
}

impl EventHandler
{
	fn new() -> Self
	{
		let (sender, recever) = mpsc::channel();
		Self
		{
			sender: Arc::new( Mutex::new(sender)),
			recever: Arc::new( Mutex::new(recever))
		}
	}

	fn new_emitter(&self, owner: UID) -> EventEmitter
	{
		EventEmitter{sender: self.sender.clone(), owner}
	}

	fn poll(&mut self, timeout: Option<Duration>) -> Result<Event, Box<dyn Error + Send>>
	{
		match timeout
		{
			Some(duration) => {
				let result = self.recever.lock().unwrap().recv_timeout(duration);

				match result {
					Ok(v) => Ok(v),
					_ => Ok(Default::default()),
				}
			}
			None => {
				let result = { self.recever.lock().unwrap().recv() };

				match result {
					Ok(v) => Ok(v),
					Err(e) => Err(Box::new(e)),
				}	
			}
		}
	}
}

#[repr(C)]
pub struct ExtEvent
{
	pub eventType: *const u8,
	pub component: UID,
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
			component: EMPTY_UID,
			arg1: 0,
			arg2: 0,
			arg3: 0
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn newEventHandler() -> *const EventHandler
{
	env_logger::init();
	
	let handler = EventHandler::new();
	let boxed = Box::new(handler);
    Box::into_raw(boxed)
}

#[no_mangle]
pub unsafe extern "C" fn startListen(hptr: *mut EventHandler, sptr: *mut ScreenComponent)
{
	let emitter = (*hptr).new_emitter((*sptr).uid());
	(*sptr).listen(Some(emitter));
}

#[no_mangle]
pub unsafe extern "C" fn pull(ptr: *mut EventHandler, t: f32) -> ExtEvent
{
	let timeout = if t > 0.0 { Some(time::Duration::from_millis((t * 1000.0) as u64))} else {None};
	
	match (*ptr).poll(timeout)
	{
		Ok(evt) => {
			return ExtEvent{
				eventType: evt.eventType.as_ptr(),
				component: evt.component,
				arg1: evt.arg1,
				arg2: evt.arg2,
				arg3: evt.arg3,
			};
		}

		Err(e) => {
			eprintln!("Event Error {}", e);
			return ExtEvent::empty();
		}
	}
}

#[no_mangle]
pub extern "C" fn create(width: u32, height: u32, fontsize: u32, handler: *mut EventHandler) -> *mut ScreenComponent
{
	let mut screen = ScreenComponent::new(width, height, fontsize);
	unsafe
	{
		if handler.is_null()
		{
			screen.set_emitter(None);
		}
		else
		{
			screen.set_emitter(Some((*handler).new_emitter(screen.uid())));
		}
	}
    Box::into_raw(Box::new(screen))
}

#[no_mangle]
pub extern "C" fn graphicHandle(width: u32, height: u32) -> *mut GraphicHandle
{
	let handle = GraphicHandle::new(width, height);
	Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub unsafe extern "C" fn bindScreen(gPtr: *mut GraphicHandle, sPtr: *mut ScreenComponent)
{
	let screen = *Box::from_raw(sPtr);
	(*gPtr).unsafe_bind_screen(screen);
}

#[no_mangle]
pub unsafe extern "C" fn destroy(ptr: *mut ScreenComponent)
{
	if !ptr.is_null() { Box::from_raw(ptr); }
}

#[no_mangle]
pub unsafe extern "C" fn foreground(ptr: *mut GraphicHandle, r: f32, g: f32, b: f32, a: f32)
{
	(*ptr).fg = Color::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn background(ptr: *mut GraphicHandle, r: f32, g: f32, b: f32, a: f32)
{
	(*ptr).bg = Color::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn fill(ptr: *const GraphicHandle, x: i32, y: i32, w: i32, h: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = &*ptr;

	handle.exec(|buffer| {
		buffer.fill(x, y, w, h, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn writeText(ptr: *mut GraphicHandle, x: i32, y: i32, cstr: *const c_char)
{
	let text = CStr::from_ptr(cstr).to_str().expect("Ungültige Zeichen");
	let handle = &*ptr;
	handle.exec(|buffer| {
		buffer.writeText(x, y, &text, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn write(ptr: *mut GraphicHandle, x: i32, y: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = &*ptr;
	handle.exec(|buffer| {
		buffer.write(x, y, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn getBuffer(ptr: *mut GraphicHandle) -> *mut Buffer
{
	println!("getBuffer");

	assert!(!ptr.is_null(), "freen is null");
	let buffer = (*ptr).get_buffer();
	Box::into_raw(Box::new(buffer))
}

#[no_mangle]
pub unsafe extern "C" fn setBuffer(ptr: *mut GraphicHandle, buf: *const Buffer)
{
	println!("setBuffer");

	assert!(!ptr.is_null(), "freen is null");
	assert!(!buf.is_null(), "buffer is null");
	(*ptr).set_buffer(&*buf);
}

#[no_mangle]
pub unsafe extern "C" fn setLocation(ptr: *mut ScreenComponent, x: i32, y: i32)
{
	assert!(!ptr.is_null(), "freen is null");
	(*ptr).screen_location(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn setSize(ptr: *mut GraphicHandle, width: u32, height: u32)
{
	assert!(!ptr.is_null(), "freen is null");
	assert!(width > 0);
	assert!(height > 0);

	(*ptr).resize_screen(width, height);
}

#[no_mangle]
pub unsafe extern "C" fn flush(ptr: *mut GraphicHandle)
{
	assert!(!ptr.is_null(), "freen is null");
	(*ptr).flush();
}

unsafe fn CStr2Char(cstr: *const c_char) -> char
{
	let str = CStr::from_ptr(cstr).to_str().expect("Ungültiges Zeichen");
	str.chars().nth(0).expect("Ungültiges Zeichen")
}
