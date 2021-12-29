#![allow(non_snake_case)]

mod component;
use crate::component::*;

mod screens;
use crate::screens::*;
use screens::screen::*;

use core::time;
use std::collections::VecDeque;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::{thread, ptr};
use std::sync::{Arc, Mutex};


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

	fn new_emitter(&self, owner: UID) -> EventEmitter
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

//const event: Event = Event{eventType: 0, component: 0};

#[no_mangle]
pub unsafe extern "C" fn newEventHandler() -> *const EventHandler
{
	let handler = EventHandler::new(32);
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
	match (*ptr).next()
	{
		Some(evt) =>
		{
			return ExtEvent{
				eventType: evt.eventType.as_ptr(),
				component: evt.component,
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
pub extern "C" fn create(width: u32, height: u32, fontsize: u32, handler: *mut EventHandler) -> *mut ScreenComponent
{
	let screen = ScreenComponent::new(width, height, fontsize);
	unsafe
	{
		if handler.is_null()
		{
			screen.open(None);
		}
		else
		{
			screen.open(Some((*handler).new_emitter(screen.uid())));
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
pub unsafe extern "C" fn destroy(ptr: *mut ScreenComponent)
{
	if !ptr.is_null() { Box::from_raw(ptr); }
}

#[no_mangle]
pub unsafe extern "C" fn foreground(ptr: *mut ScreenComponent, r: f32, g: f32, b: f32, a: f32)
{
	(*ptr).fg = Color::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn background(ptr: *mut ScreenComponent, r: f32, g: f32, b: f32, a: f32)
{
	(*ptr).bg = Color::new(r, g, b, a);
}

#[no_mangle]
pub unsafe extern "C" fn fill(ptr: *const ScreenComponent, x: i32, y: i32, w: i32, h: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = &*ptr;

	handle.withBuffer(|buffer| {
		buffer.fill(x, y, w, h, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn writeText(ptr: *mut ScreenComponent, x: i32, y: i32, cstr: *const c_char)
{
	let text = CStr::from_ptr(cstr).to_str().expect("Ungültige Zeichen");
	let handle = &*ptr;
	handle.withBuffer(|buffer| {
		buffer.writeText(x, y, &text, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn write(ptr: *mut ScreenComponent, x: i32, y: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = &*ptr;
	handle.withBuffer(|buffer| {
		buffer.write(x, y, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn setSize(ptr: *mut ScreenComponent, width: u32, height: u32)
{
	assert!(width > 0);
	assert!(height > 0);

	(*ptr).resize_screen(width, height);
}

#[no_mangle]
pub unsafe extern "C" fn flush(ptr: *mut ScreenComponent)
{
	(*ptr).flush();
}

unsafe fn CStr2Char(cstr: *const c_char) -> char
{
	let str = CStr::from_ptr(cstr).to_str().expect("Ungültiges Zeichen");
	str.chars().nth(0).expect("Ungültiges Zeichen")
}
