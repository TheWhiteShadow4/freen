#![allow(non_snake_case)]

mod component;
use crate::component::*;

mod events;
use crate::events::*;

mod files;
use crate::files::Filesystem;

mod network;
use crate::network::*;

mod screens;
use crate::screens::*;
use screens::screen::*;

use core::time;
use std::ffi::CStr;
use std::os::raw::c_char;


unsafe fn param_to_vec(data: *const C_Param, len: usize) -> Vec<String>
{
	let mut vec = Vec::<String>::with_capacity(len);
	let mut ptr = data;
	for _i in 0..len
	{
		let slice = ptr.read();
		ptr = ptr.offset(1);
		vec.push(slice.to_string());
	}
	vec
}

#[inline]
unsafe fn handle<T>(h: *mut T) -> &'static mut T
{
	assert!(!h.is_null(), "handle is null");
	&mut *h
}

#[inline]
unsafe fn c2str(cstr: *const c_char) -> &'static str
{
	if cstr.is_null() { panic!("null pointer!"); }
	CStr::from_ptr(cstr).to_str().expect("Ungültiges Zeichen")
}

#[inline]
unsafe fn c2char(ch: *const c_char) -> char
{
	c2str(ch).chars().nth(0).expect("Ungültiges Zeichen")
}

// Event Testing Funktion
#[no_mangle]
pub unsafe extern "C" fn event_test(handler: *mut EventHandler, len: usize, data: *const C_Param)
{
	println!("Länge {}", len);

	let vector = param_to_vec(data, len);

	//let mut ptr = data;
	for s in vector
	{
		//let slice = ptr.read();
		//let s = std::str::from_utf8(slice.into()).unwrap();
		//ptr = ptr.offset(1);
		println!("String: {}", s);
	}

	let arg1: C_Param;
	let arg2: C_Param;
	{
		let str = "Ausgabe Arg 1";
		arg1 = C_Param::from_str(str);

		let str = "Ausgabe Argument 2";
		arg2 = C_Param::from_str(str);
	}

	let signal = Signal::raw("Text Event Type\0", generateUID(), vec![arg1, arg2]);
	handle(handler).sender().lock().unwrap().send(signal).ok();

	//let str = "Ausgabe Text\0".to_string();

	//let ptr = Box::into_raw(Box::new(str.as_bytes())) as *const u8;
	//let slice = C_Slice {ptr, len: 12};

	//let mut slice = out.read();
	//slice.ptr = ptr;

	/*let s = slice::from_raw_parts(data, len1).clone().as_ptr() as *mut c_char;
	let str = CString::from_raw(s);
	let ptr = str.as_ptr();// slice::from_raw_parts(data, len1);
	println!("{}", ptr as usize)*/

	//str1.replace();
	//let n = str1.offset(len1 - 1);
	//n.write(65);
	
}

/*
#[no_mangle]
pub unsafe extern "C" fn signal_arg(ptr: *const C_Param, idx: isize) -> C_Param
{
	ptr.offset(idx).read()
}
*/

#[no_mangle]
pub unsafe extern "C" fn new_event_handler() -> *const EventHandler
{
	env_logger::try_init().ok();
    Box::into_raw(Box::new(EventHandler::new()))
}

#[no_mangle]
pub unsafe extern "C" fn start_listen(hptr: *mut EventHandler, sptr: *mut ScreenComponent)
{
	let screen = handle(sptr);
	let emitter = handle(hptr).new_emitter(screen.uid());
	screen.listen(Some(emitter));
}

#[no_mangle]
pub unsafe extern "C" fn pull(ptr: *mut EventHandler, t: f32) -> Signal
{
	let timeout = if t > 0.0 { Some(time::Duration::from_millis((t * 1000.0) as u64))} else {None};

	let handler = handle(ptr);
	match handler.poll(timeout)
	{
		Ok(sig) => { return sig; }

		Err(e) => {
			eprintln!("Event Error {}", e);
			return Signal::default();
		}
	}
}

#[no_mangle]
pub extern "C" fn create_screen(fontsize: u32, handler: *mut EventHandler) -> UIDHandle<ScreenComponent>
{
	assert!(fontsize > 1);
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
pub unsafe extern "C" fn destroy_screen(ptr: *mut ScreenComponent)
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
pub unsafe extern "C" fn fill(ptr: *mut GraphicHandle, x: i32, y: i32, w: i32, h: i32, ch: *const c_char)
{
	let handle = handle(ptr);
	handle.exec(|buffer| {
		buffer.fill(x, y, w, h, c2char(ch), handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn write_text(ptr: *mut GraphicHandle, x: i32, y: i32, cstr: *const c_char)
{
	let handle = handle(ptr);
	handle.exec(|buffer| {
		buffer.writeText(x, y, &c2str(cstr), handle.fg, handle.bg);
	});
}

#[no_mangle]
pub unsafe extern "C" fn write(ptr: *mut GraphicHandle, x: i32, y: i32, ch: *const c_char)
{
	let handle = handle(ptr);
	handle.exec(|buffer| {
		buffer.write(x, y, c2char(ch), handle.fg, handle.bg);
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
pub unsafe extern "C" fn buf_fill(ptr: *mut Buffer, x: i32, y: i32, w: i32, h: i32, ch: *const c_char, fg: Color, bg: Color)
{
	handle(ptr).fill(x, y, w, h, c2char(ch), fg.clone(), bg.clone());
}

#[no_mangle]
pub unsafe extern "C" fn buf_write(ptr: *mut Buffer, x: i32, y: i32, cstr: *const c_char, fg: Color, bg: Color)
{
	handle(ptr).writeText(x, y, c2str(cstr), fg.clone(), bg.clone());
}

#[no_mangle]
pub unsafe extern "C" fn buf_set(ptr: *mut Buffer, x: i32, y: i32, ch: *const c_char, fg: Color, bg: Color)
{
	handle(ptr).write(x, y, c2char(ch), fg.clone(), bg.clone());
}

#[no_mangle]
pub unsafe extern "C" fn buf_get(ptr: *mut Buffer, x: u32, y: u32) -> BufferCell
{
	let buffer = handle(ptr);
	let idx = x as usize + y as usize * buffer.width as usize;

	let mut char = [0; 4];
	let len = buffer.chars[idx].encode_utf8(&mut char).len();

	BufferCell{
		char,
		len,
		fg: buffer.foreground[idx].clone(),
		bg: buffer.background[idx].clone(),
	}
}

#[no_mangle]
pub extern "C" fn create_network(port_start: u16, buffer_size: usize, handler: *mut EventHandler) -> UIDHandle<NetworkComponent>
{
	let mut network = NetworkComponent::new(port_start, buffer_size);
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
//pub unsafe extern "C" fn send_message(ptr: *mut NetworkComponent, reciever: *const c_char, port: u16, data: *const u8, len: usize)
pub unsafe extern "C" fn send_message(ptr: *mut NetworkComponent, reciever: *const c_char, port: u16, data: *const C_Param, len: usize)
{
	handle(ptr).send(c2str(reciever), port, param_to_vec(data, len));
}

#[no_mangle]
//pub unsafe extern "C" fn broadcast_message(ptr: *mut NetworkComponent, port: u16, data: *const u8, len: usize)
pub unsafe extern "C" fn broadcast_message(ptr: *mut NetworkComponent, port: u16, data: *const C_Param, len: usize)
{
	handle(ptr).broadcast(port, param_to_vec(data, len));
}


#[no_mangle]
pub unsafe extern "C" fn create_filesystem(root_path: *const c_char, name: *const c_char, handler: *mut EventHandler) -> *mut Filesystem
{
	let mut fs = Filesystem::new(c2str( root_path), c2str(name));
	if handler.is_null()
	{
		fs.listen(None);
	}
	else
	{
		fs.listen(Some((*handler).new_emitter(UID::default())));
	}
	Box::into_raw(Box::new(fs))
}

#[no_mangle]
pub unsafe extern "C" fn mount(ptr: *mut Filesystem, c_device: *const c_char, c_mount: *const c_char) -> bool
{
	handle(ptr).mount(c2str(c_device), c2str(c_mount))
}

#[no_mangle]
pub unsafe extern "C" fn unmount(ptr: *mut Filesystem, c_device: *const c_char) -> bool
{
	handle(ptr).unmount(c2str(c_device))
}

#[no_mangle]
pub unsafe extern "C" fn fs_exists(ptr: *mut Filesystem, c_path: *const c_char) -> bool
{
	handle(ptr).exists(c2str(c_path))
}

#[no_mangle]
pub unsafe extern "C" fn fs_is_file(ptr: *mut Filesystem, c_path: *const c_char) -> bool
{
	handle(ptr).is_file(c2str(c_path))
}

#[no_mangle]
pub unsafe extern "C" fn fs_is_dir(ptr: *mut Filesystem, c_path: *const c_char) -> bool
{
	handle(ptr).is_dir(c2str(c_path))
}

#[no_mangle]
pub unsafe extern "C" fn fs_remove(ptr: *mut Filesystem, c_path: *const c_char) -> bool
{
	handle(ptr).remove(c2str(c_path))
}

#[no_mangle]
pub unsafe extern "C" fn fs_rename(ptr: *mut Filesystem, c_from: *const c_char, c_to: *const c_char) -> bool
{
	handle(ptr).rename(c2str(c_from),c2str(c_to))
}

#[no_mangle]
pub unsafe extern "C" fn fs_create_dir(ptr: *mut Filesystem, c_path: *const c_char) -> bool
{
	handle(ptr).create_dir(c2str(c_path))
}

#[no_mangle]
pub unsafe extern "C" fn fs_childs(ptr: *mut Filesystem, c_path: *const c_char) -> C_Array
{
	let vec = handle(ptr).childs(c2str(c_path));
	C_Array::new(&vec)
}