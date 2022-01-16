#![allow(non_snake_case)]

use core::panic;
use std::error::Error;
use std::ffi::CString;
use std::ptr;
use std::slice;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use crate::component::UID;


pub struct EventEmitter
{
	sender: Arc<Mutex<mpsc::Sender<Signal>>>,
	owner: UID
}

impl EventEmitter
{
	pub fn send(&mut self, event: Signal)
	{
		if let Err(e) = self.sender.lock().unwrap().send(event)
		{
			eprintln!("Event Error {}", e);
		}
	}

	pub fn owner(&self) -> UID { self.owner }
}

pub struct EventHandler
{
	sender: Arc<Mutex<mpsc::Sender<Signal>>>,
	recever: Arc<Mutex<mpsc::Receiver<Signal>>>
}

impl EventHandler
{
	pub fn new() -> Self
	{
		let (sender, recever) = mpsc::channel();
		Self
		{
			sender: Arc::new( Mutex::new(sender)),
			recever: Arc::new( Mutex::new(recever))
		}
	}

	pub fn sender(&self) -> &Arc<Mutex<mpsc::Sender<Signal>>>
	{
		&self.sender
	}

	pub fn new_emitter(&self, owner: UID) -> EventEmitter
	{
		EventEmitter{sender: self.sender.clone(), owner}
	}

	pub fn poll(&mut self, timeout: Option<Duration>) -> Result<Signal, Box<dyn Error + Send>>
	{
		match timeout
		{
			Some(duration) => {
				let result = self.recever.lock().unwrap().recv_timeout(duration);

				match result {
					Ok(v) => Ok(v),
					_ => Ok(Signal::default()),
				}
			}
			None => {
				let result = { self.recever.lock().unwrap().try_recv() };

				match result {
					Ok(v) => Ok(v),
					_ => Ok(Signal::default()),
				}	
			}
		}
	}
}

#[repr(C)]
pub struct C_Array
{
	pub ptr: *const *const u8,
	pub len: usize
}

impl C_Array
{
	pub fn new<T: AsRef<str>>(array: &[T]) -> Self
	{
		let mut result = Vec::<*const u8>::with_capacity(array.len());
		let len = array.len();
		for s in array
		{
			let str = s.as_ref().to_owned();
			result.push(str.as_ptr());
			std::mem::forget(str);
		}
		let ptr = result.as_ptr();
		std::mem::forget(result);
		Self { ptr, len }
	}

	pub unsafe fn get(&self, i: isize) -> String
	{
		let ptr = self.ptr.offset(i).read();
		CString::from_raw(ptr as *mut i8).into_string().unwrap()
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct C_Param
{
	pub ptr: *const u8,
	pub len: usize,
	pub is_num: bool
}

impl C_Param
{
	pub fn from_str(str: &str) -> Self
	{
		let len = str.len();
		let ptr = str.as_bytes().as_ptr();
		std::mem::forget(str);

		Self { ptr, len, is_num: false }
	}

	pub fn from<T>(v: &T, is_num: bool) -> Self
	where T: ToString
	{
		let str = v.to_string();
		let len = str.len();
		let ptr = str.as_bytes().as_ptr();
		std::mem::forget(str);

		Self { ptr, len, is_num }
	}

	#[inline]
	pub fn default() -> Self
	{
		Self { ptr: ptr::null(), len: 0, is_num: false }
	}

	pub fn to_string(self) -> String
	{
		unsafe { String::from_raw_parts(self.ptr as *mut u8, self.len, self.len) }
	}

	#[inline]
	pub fn as_str(self) -> &'static str
	{
		std::str::from_utf8(self.into()).unwrap()
	}
}

impl Into<&[u8]> for C_Param
{
	#[inline]
	fn into(self) -> &'static [u8]
	{
		unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}


#[repr(C)]
pub struct Signal
{
	pub eventType: *const u8,
	pub component: UID,
	pub len: usize,
	pub args: [C_Param; 8],
}

unsafe impl Send for Signal {}
unsafe impl Sync for Signal {}

impl Signal
{
	pub fn numArgs<T>(eventType: &'static str, component: UID, vec: Vec<T>) -> Self
	where T: ToString
	{
		let mut args = [C_Param::default(); 8];
		for i in 0..vec.len().min(8)
		{
			args[i] = C_Param::from(&vec[i], true);
		}

		Self
		{
			eventType: eventType.as_ptr(),
			component,
			len: vec.len(),
			args: args
		}
	}

	pub fn raw(eventType: &'static str, component: UID, params: Vec<C_Param>) -> Self
	{
		let mut args = [C_Param::default(); 8];
		let len = params.len().min(8);
		args[0..len].copy_from_slice(&params[0..len]);

		Self
		{
			eventType: eventType.as_ptr(),
			component,
			len: params.len(),
			args: args
		}
	}

	pub fn default() -> Self
	{
		Self
		{
			eventType: ptr::null(),
			component: UID::default(),
			len: 0,
			args: [C_Param::default(); 8]
		}
	}
	
	pub fn noArgs(eventType: &'static str, comp: UID) -> Self
	{
		Self
		{
			eventType: eventType.as_ptr(),
			component: comp,
			len: 0,
			args: [C_Param::default(); 8]
		}
	}

	pub fn name(&self) -> &'static str
	{
		if self.eventType.is_null() { panic!("invalid event!") }
		let ptr = self.eventType as *const &str;
		unsafe { ptr.read() }
	}
}