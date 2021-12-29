use rand::Rng;

pub type UID = u64;

pub static EMPTY_UID: UID = 0;

pub trait Component
{
	fn uid(&self) -> UID;
}

pub struct GenericComponent
{
	id: UID
}

impl Component for GenericComponent
{
    fn uid(&self) -> UID
	{
		self.id
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