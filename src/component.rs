use rand::distributions::{Distribution, Uniform};

use crate::{EventEmitter, Event};

pub type UID = [u8; 16];

//pub static EMPTY_UID: UID = 0;

#[repr(C)]
pub struct UIDHandle<T>
{
	pub id: UID,
	pub handle: *mut T,
}

impl<T: Component> UIDHandle<T>
{
	pub fn new(val: T) -> Self
	{
		Self{ id: val.uid(), handle: Box::into_raw(Box::new(val)) }
	}
}

pub trait Component
{
	fn uid(&self) -> UID;
	fn listen(&mut self, emitter: Option<EventEmitter>);
}

pub struct GenericComponent
{
	id: UID,
	emitter: Option<EventEmitter>,
}

impl GenericComponent
{
	#![allow(dead_code)]
	pub fn fire_event(&mut self, eventType: &'static str, arg1: i32, arg2: i32, arg3: i32, arg4: usize)
	{
		if self.emitter.is_none() {return;}
		self.emitter.as_mut().unwrap().send(Event::new(eventType, self.id, arg1, arg2, arg3, arg4));
	}
}

impl Component for GenericComponent
{
    fn uid(&self) -> UID
	{
		self.id
    }

    fn listen(&mut self, emitter: Option<EventEmitter>)
	{
        self.emitter = emitter;
    }
}

const HEX_DIGITS: *const u8 = "0123456789ABCDEF".as_ptr();
pub fn generateUID() -> UID
{
	let mut rng = rand::thread_rng();
	let hex: Uniform<isize> = Uniform::from(1..16);

	let mut id = [0; 16];
	unsafe
	{
		for i in 0..16
		{
			id[i] = *HEX_DIGITS.offset(hex.sample(&mut rng)) as u8;
		}
	}
	id
}