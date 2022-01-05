#![allow(non_snake_case)]

mod component;
use crate::component::*;

mod network;
use crate::network::*;

mod screens;
use crate::screens::*;
use screens::screen::*;

use core::panic;
use core::time;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::{Arc, Mutex};

const EVENT_NETWORK_MESSAGE: &str = "NetworkMessage\0";
const EVENT_WINDOW_CLOSED: &str = "WindowClosed\0";
const EVENT_MOUSE_DOWN: &str = "OnMouseDown\0";
const EVENT_MOUSE_UP: &str = "OnMouseUp\0";
const EVENT_MOUSE_MOVE: &str = "OnMouseMove\0";
const EVENT_KEY_DOWN: &str = "OnKeyDown\0";
const EVENT_KEY_UP: &str = "OnKeyUp\0";

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
	extra_data: Option<Box<[u8; 1<<16]>>
}

impl EventHandler
{
	fn new() -> Self
	{
		let (sender, recever) = mpsc::channel();
		Self
		{
			sender: Arc::new( Mutex::new(sender)),
			recever: Arc::new( Mutex::new(recever)),
			extra_data: None
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
				let result = { self.recever.lock().unwrap().try_recv() };

				match result {
					Ok(v) => Ok(v),
					Err(e) => Err(Box::new(e)),
				}	
			}
		}
	}
}

pub type EventExtra = [u8; 1<<16];

#[repr(C)]
#[derive(Default)]
pub struct Event
{
	pub eventType: usize,
	pub component: UID,
	pub arg1: i32,
	pub arg2: i32,
	pub arg3: i32,
	pub extra: usize
}

impl Event
{
	pub fn new(eventType: &'static str, comp: UID, arg1: i32, arg2: i32, arg3: i32, extra: usize) -> Self
	{
		Self
		{
			eventType: eventType.as_ptr() as usize,
			component: comp,
			arg1,
			arg2,
			arg3,
			extra
		}
	}

	pub fn noArgs(eventType: &'static str, comp: UID) -> Self
	{
		Self
		{
			eventType: eventType.as_ptr() as usize,
			component: comp,
			arg1: 0,
			arg2: 0,
			arg3: 0,
			extra: 0
		}
	}

	pub fn name(&self) -> &'static str
	{
		if self.eventType == 0 { panic!("invalid event!") }
		let ptr = self.eventType as *const &str;
		unsafe { ptr.read() }
	}
}

#[inline]
unsafe fn handle<T>(h: *mut T) -> &'static mut T
{
	assert!(!h.is_null(), "handle is null");
	&mut *h
}

unsafe fn CStr2Char(cstr: *const c_char) -> char
{
	let str = CStr::from_ptr(cstr).to_str().expect("Ungültiges Zeichen");
	str.chars().nth(0).expect("Ungültiges Zeichen")
}

/*unsafe fn convertData(ImageDataLayout: *const u8) -> [u8]
{
	return [u8; 1]
}*/

#[no_mangle]
pub unsafe extern "C" fn new_event_handler() -> *const EventHandler
{
	env_logger::try_init().ok();
	
	let handler = EventHandler::new();
	let boxed = Box::new(handler);
    Box::into_raw(boxed)
}

#[no_mangle]
pub unsafe extern "C" fn start_listen(hptr: *mut EventHandler, sptr: *mut ScreenComponent)
{
	let screen = handle(sptr);
	let emitter = handle(hptr).new_emitter(screen.uid());
	screen.listen(Some(emitter));
}

#[no_mangle]
pub unsafe extern "C" fn pull(ptr: *mut EventHandler, t: f32) -> Event
{
	let timeout = if t > 0.0 { Some(time::Duration::from_millis((t * 1000.0) as u64))} else {None};
	
	let handler = handle(ptr);
	match handler.poll(timeout)
	{
		Ok(evt) => {
			if evt.extra != 0
			{
				handler.extra_data = Some(Box::from_raw(evt.extra as *mut EventExtra));
			}
			return evt;
		}

		Err(e) => {
			eprintln!("Event Error {}", e);
			return Event::default();
		}
	}
}

#[no_mangle]
pub extern "C" fn create_screen(fontsize: u32, handler: *mut EventHandler) -> UIDHandle<ScreenComponent>
{
	let mut screen = ScreenComponent::new(fontsize);
	unsafe
	{
		if handler.is_null()
		{
			screen.listen(None);
		}
		else
		{
			screen.listen(Some((*handler).new_emitter(screen.uid())));
		}
	}
	UIDHandle::new(screen)
}

#[no_mangle]
pub extern "C" fn graphic_handle(width: u32, height: u32) -> *mut GraphicHandle
{
	let gpu = GraphicHandle::new(width, height);
	Box::into_raw(Box::new(gpu))
}

#[no_mangle]
pub unsafe extern "C" fn bind_screen(gPtr: *mut GraphicHandle, sPtr: *mut ScreenComponent)
{
	if sPtr.is_null()
	{
		handle(gPtr).bind_screen(None);
	}
	else
	{
		let screen = *Box::from_raw(sPtr);
		handle(gPtr).bind_screen(Some(screen));
	}
}

#[no_mangle]
pub unsafe extern "C" fn destroy(ptr: *mut ScreenComponent)
{
	if !ptr.is_null() { Box::from_raw(ptr); }
}

#[no_mangle]
pub unsafe extern "C" fn foreground(ptr: *mut GraphicHandle, col: Color)
{
	handle(ptr).fg = col;
}

#[no_mangle]
pub unsafe extern "C" fn background(ptr: *mut GraphicHandle, col: Color)
{
	handle(ptr).bg = col;
}

#[no_mangle]
pub unsafe extern "C" fn fill(ptr: *mut GraphicHandle, x: i32, y: i32, w: i32, h: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = handle(ptr);

	handle.exec(|buffer| {
		buffer.fill(x, y, w, h, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn write_text(ptr: *mut GraphicHandle, x: i32, y: i32, cstr: *const c_char)
{
	let text = CStr::from_ptr(cstr).to_str().expect("Ungültige Zeichen");
	let handle = handle(ptr);
	handle.exec(|buffer| {
		buffer.writeText(x, y, &text, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn write(ptr: *mut GraphicHandle, x: i32, y: i32, cstr: *const c_char)
{
	let char = CStr2Char(cstr);
	let handle = handle(ptr);
	handle.exec(|buffer| {
		buffer.write(x, y, char, handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn set_location(ptr: *mut ScreenComponent, x: i32, y: i32)
{
	handle(ptr).screen_location(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn set_size(ptr: *mut GraphicHandle, width: u32, height: u32)
{
	assert!(width > 0);
	assert!(height > 0);

	handle(ptr).resize_screen(width, height);
}

#[no_mangle]
pub unsafe extern "C" fn flush(ptr: *mut GraphicHandle)
{
	handle(ptr).flush();
}

#[no_mangle]
pub unsafe extern "C" fn get_buffer(ptr: *mut GraphicHandle) -> *mut Buffer
{
	let buffer = handle(ptr).get_buffer();
	Box::into_raw(Box::new(buffer))
}

#[no_mangle]
pub unsafe extern "C" fn set_buffer(ptr: *mut GraphicHandle, buf: *mut Buffer)
{
	handle(ptr).set_buffer(handle(buf));
}

#[no_mangle]
pub unsafe extern "C" fn buf_size(ptr: *mut Buffer) -> Size
{
	let buffer = handle(ptr);
	Size{width: buffer.width, height: buffer.height}
}

#[no_mangle]
pub unsafe extern "C" fn buf_resize(ptr: *mut Buffer, width: u32, height: u32)
{
	handle(ptr).resize(width, height);
}

#[no_mangle]
pub unsafe extern "C" fn buf_copy(ptr: *mut Buffer, x: i32, y: i32, other: *mut Buffer, txtbm: u8, fgbm: u8, bgbm: u8)
{
	handle(ptr).copy(x, y, handle(other), txtbm, fgbm, bgbm);
}

#[no_mangle]
pub unsafe extern "C" fn buf_clone(ptr: *mut Buffer) -> *mut Buffer
{
	Box::into_raw(Box::new(handle(ptr).clone()))
}

#[no_mangle]
pub unsafe extern "C" fn buf_fill(ptr: *mut Buffer, x: i32, y: i32, w: i32, h: i32, cstr: *const c_char, fg: Color, bg: Color)
{
	let char = CStr2Char(cstr);
	handle(ptr).fill(x, y, w, h, char, fg.clone(), bg.clone());
}

#[no_mangle]
pub unsafe extern "C" fn buf_write(ptr: *mut Buffer, x: i32, y: i32, cstr: *const c_char, fg: Color, bg: Color)
{
	let text = CStr::from_ptr(cstr).to_str().expect("Ungültige Zeichen");
	handle(ptr).writeText(x, y, text, fg.clone(), bg.clone());
}

#[no_mangle]
pub unsafe extern "C" fn buf_set(ptr: *mut Buffer, x: i32, y: i32, cstr: *const c_char, fg: Color, bg: Color)
{
	let char = CStr2Char(cstr);
	handle(ptr).write(x, y, char, fg.clone(), bg.clone());
}

#[no_mangle]
pub unsafe extern "C" fn buf_get(ptr: *mut Buffer, x: u32, y: u32) -> BufferCell
{
	let buffer = handle(ptr);
	let idx = x as usize + y as usize * buffer.width as usize;
	let char_ptr = Box::into_raw(Box::new(buffer.chars[idx].to_string()));
	//let char_ptr = buffer.chars[idx].to_string().as_ptr();

	BufferCell{
		char: char_ptr,
		fg: buffer.foreground[idx].clone(),
		bg: buffer.background[idx].clone(),
	}
}

#[no_mangle]
pub extern "C" fn create_network(port_start: u16, handler: *mut EventHandler) -> UIDHandle<NetworkComponent>
{
	let mut network = NetworkComponent::new(port_start);
	unsafe
	{
		if handler.is_null()
		{
			network.listen(None);
		}
		else
		{
			network.listen(Some((*handler).new_emitter(network.uid())));
		}
	}
	UIDHandle::new(network)
}

#[no_mangle]
pub unsafe extern "C" fn open_port(ptr: *mut NetworkComponent, port: u16) -> bool
{
	handle(ptr).open_port(port)
}

#[no_mangle]
pub unsafe extern "C" fn close_port(ptr: *mut NetworkComponent, port: u16)
{
	handle(ptr).close_port(port);
}

#[no_mangle]
pub unsafe extern "C" fn close_all_ports(ptr: *mut NetworkComponent)
{
	handle(ptr).close_all();
}

#[no_mangle]
pub unsafe extern "C" fn send_message(ptr: *mut NetworkComponent, reciever: *const c_char, port: u16, data: *const u8, len: usize)
{
	let reciever_str = CStr::from_ptr(reciever).to_str().expect("Ungültige Zeichen");
	let buf = slice::from_raw_parts(data, len);
	handle(ptr).send(reciever_str, port, buf);
}

#[no_mangle]
pub unsafe extern "C" fn broadcast_message(ptr: *mut NetworkComponent, port: u16, data: *const u8, len: usize)
{
	let buf = slice::from_raw_parts(data, len);
	handle(ptr).broadcast(port, buf);
}