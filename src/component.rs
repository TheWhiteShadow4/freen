#![allow(non_snake_case)]

use rand::distributions::{Distribution, Uniform};

use crate::events::EventEmitter;

pub const UID_SIZE: usize = 16;
pub type UID = [u8; UID_SIZE];

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
	let hex: Uniform<isize> = Uniform::from(0..16);

	let mut id = [0; UID_SIZE];
	unsafe
	{
		for i in 0..UID_SIZE
		{
			id[i] = *HEX_DIGITS.offset(hex.sample(&mut rng)) as u8;
		}
	}
	id
}