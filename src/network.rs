
use core::slice;
use std::{net::{Ipv4Addr, SocketAddrV4, UdpSocket}, thread, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, collections::HashMap, alloc::Layout};

use nanoserde::SerBin;
use rand::Rng;

use crate::{EventEmitter, component::{UID, Component, generateUID, UID_SIZE}, Signal, events::C_Param};

const EVENT_NETWORK_MESSAGE: &str = "NetworkMessage\0";

struct SocketListener
{
	pub socket: UdpSocket,
	pub active: AtomicBool
}

pub struct NetworkComponent
{
	id: UID,
	port_Start: u16,
	buffer_size: usize,
	sockets: HashMap<u16, Arc<SocketListener>>,
	sender_socket: Option<UdpSocket>,
	emitter: Arc<Mutex<Option<EventEmitter>>>,
}

impl NetworkComponent
{
	pub fn new(port_Start: u16, buffer_size: usize) -> Self
	{
		Self
		{
			id: generateUID(),
			port_Start,
			buffer_size,
			sockets: HashMap::new(),
			sender_socket: None,
			emitter: Arc::default(),
		}
	}


	pub fn open_port(&mut self, port: u16) -> bool
	{
		if self.emitter.lock().unwrap().is_none() { return false; }

		let port_Start = self.port_Start;
		let buffer_size = self.buffer_size;
		let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port + port_Start);
		match UdpSocket::bind(addr)
		{
			Ok(socket) => {
				let listener =  Arc::new(SocketListener{socket, active: AtomicBool::new(true)});
				self.sockets.insert(port, listener.clone());
				let emitter = self.emitter.clone();
				thread::spawn(move || {
					let mut buf = alloc_buffer(buffer_size);
					while listener.active.load(Ordering::Relaxed) == true
					{
						match listener.socket.recv_from(&mut buf)
						{
							Ok((_size, _addr)) => {
								let sender_uid = uid_from_buffer(&buf, 0);

								let mut offset: usize = UID_SIZE;
								let data: Vec<String> = nanoserde::DeBin::de_bin(&mut offset, &buf).unwrap();

								let mut params = Vec::<C_Param>::new();
								params.push(C_Param::from_str(&sender_uid));
								params.push(C_Param::from(&port, true));

								data.iter().for_each(|d| params.push(C_Param::from_str(&d)));

								let mut lock = emitter.lock().unwrap();
								let e = lock.as_mut().unwrap();
								
								let signal = Signal::raw(EVENT_NETWORK_MESSAGE, e.owner(), params);

								/*println!("len: {}", signal.len);
								for i in 0..(signal.len)
								{
									let param = signal.args[i];
									println!("offset: {} len {}", i, param.len);
									//std::slice::from_raw_parts(param.ptr, param.len).;

									println!("Daten: {}: {}", i, param.ptr as usize);
								}*/

								e.send(signal);
							},
							Err(e) => { eprintln!("{}", e); }
						}
					}
					drop_buffer(&mut buf, buffer_size);
				});
				true
			},
			Err(_e) => false
		}
	}

	pub fn close_port(&mut self, port: u16)
	{
		if let Some(l) = self.sockets.get(&port)
		{
			l.active.store(false, Ordering::Relaxed);
			l.socket.set_nonblocking(true).ok();
		}
		self.sockets.remove(&port);
	}

	pub fn close_all(&mut self)
	{
		self.sockets.retain( |_k, l| {
			l.active.store(false, Ordering::Relaxed);
			l.socket.set_nonblocking(true).ok();
			false
		});

		self.sockets.clear();
	}

	pub fn send(&mut self, _reciever: &str, port: u16, data: Vec::<String>)
	{
		if self.sender_socket.is_none()
		{
			self.create_sender_socket();
		}
		
		if let Some(socket) = &self.sender_socket
		{
			let rl_port = port + self.port_Start;
			let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, rl_port);

			let mut buffer = Vec::<u8>::default();
			buffer.append(&mut self.id.to_vec());
			data.ser_bin(&mut buffer);

			let result = socket.send_to(&buffer, addr);
			match result
			{
				Ok(_len) => {},
				Err(e) => eprintln!("{:?}", e)
			}
		}
	}

	fn create_sender_socket(&mut self)
	{
		let mut rng = rand::thread_rng();
		let send_port = rng.gen_range(30001..u16::MAX);
		let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, send_port);
		self.sender_socket = UdpSocket::bind(addr).ok();
	}

	pub fn broadcast(&mut self, port: u16, data: Vec::<String>)
	{
		self.send("", port, data);
	}
}

impl Component for NetworkComponent
{
	fn uid(&self) -> UID { self.id }

	fn listen(&mut self, emitter: Option<EventEmitter>)
	{
		self.emitter = Arc::new(Mutex::new(emitter));
    }
}

fn alloc_buffer(size: usize) -> &'static mut [u8]
{
	unsafe
	{
		let layout = Layout::array::<u8>(size).expect("Invalid Buffer size.");
		let a = std::alloc::alloc(layout);
		slice::from_raw_parts_mut(a, size)
	}
}

fn drop_buffer(buffer: &mut [u8], size: usize)
{
	unsafe
	{
		let layout = Layout::array::<u8>(size).expect("Invalid Buffer size.");
		std::alloc::dealloc(buffer.as_mut_ptr(), layout);
	}
}

#[inline]
fn uid_from_buffer(buf: &[u8], off: usize) -> String
{
	unsafe { String::from_utf8_unchecked(buf[off..off+UID_SIZE].to_owned()) }
}