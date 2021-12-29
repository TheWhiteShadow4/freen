use rand::Rng;

use crate::{EventEmitter, Event};

pub type UID = u64;

pub static EMPTY_UID: UID = 0;

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
	pub fn fire_event(&mut self, eventType: String, arg1: i32, arg2: i32, arg3: i32)
	{
		if self.emitter.is_none() {return;}
		self.emitter.as_mut().unwrap().send(Event{eventType, component: self.id, arg1, arg2, arg3});
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

pub fn generateUID() -> UID
{
	let mut rng = rand::thread_rng();
	//let n = rng.gen::<u64>();
	//let ptr: usize = unsafe { std::mem::transmute(n) };
	//CStr::from_ptr(ptr);

	
	//println!("Magic {}", ptr);

	//std::mem::forget(*n);
	//let ptr = *n;  //[rng.gen(), rng.gen(), rng.gen(), rng.gen()]
	rng.gen::<u64>()
}