
use std::{net::{Ipv4Addr, SocketAddrV4, UdpSocket}, thread, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, collections::HashMap};

use rand::Rng;

use crate::{EventEmitter, component::{UID, Component, generateUID}, Event, EVENT_NETWORK_MESSAGE, EventExtra};

struct SocketListener
{
	pub socket: UdpSocket,
	pub active: AtomicBool
}

pub struct NetworkComponent
{
	id: UID,
	port_Start: u16,
	sockets: HashMap<u16, Arc<SocketListener>>,
	sender_socket: Option<UdpSocket>,
	emitter: Arc<Mutex<Option<EventEmitter>>>,
}

impl NetworkComponent
{
	pub fn new(port_Start: u16) -> Self
	{
		Self
		{
			id: generateUID(),
			port_Start,
			sockets: HashMap::new(),
			sender_socket: None,
			emitter: Arc::default(),
		}
	}

	pub fn open_port(&mut self, port: u16) -> bool
	{
		if self.emitter.lock().unwrap().is_none() { return false; }

		let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port + self.port_Start);
		match UdpSocket::bind(addr)
		{
			Ok(socket) => {
				let listener =  Arc::new(SocketListener{socket, active: AtomicBool::new(true)});
				self.sockets.insert(port, listener.clone());
				let emitter = self.emitter.clone();
				thread::spawn(move || {
					let mut buf: EventExtra = [0; 1<<16];
					while listener.active.load(Ordering::Relaxed) == true
					{
						match listener.socket.recv_from(&mut buf)
						{
							Ok((bytes, _addr)) => {
								//println!("Empfange {} bytes von {}", bytes, addr.port());
								let mut lock = emitter.lock().unwrap();
								let e = lock.as_mut().unwrap();
								let ptr = Box::into_raw(Box::new(buf)) as usize;
								e.send(Event::new(EVENT_NETWORK_MESSAGE, e.owner(), port.into(), 0, bytes.try_into().unwrap(), ptr));
							},
							Err(e) => { eprintln!("{}", e); }
						}
					}
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

	pub fn send(&mut self, _reciever: &str, port: u16, buf: &[u8])
	{
		//println!("Sende zu {}", port);

		if self.sender_socket.is_none()
		{
			self.create_sender_socket();
		}
		
		if let Some(socket) = &self.sender_socket
		{
			let rl_port = port + self.port_Start;
			let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, rl_port);
			let result = socket.send_to(buf, addr);
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

	pub fn broadcast(&mut self, port: u16, buf: &[u8])
	{
		self.send("", port, buf);
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