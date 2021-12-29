use rand::Rng;


pub type UID = [u32; 4];

pub static EMPTY_UID: UID = [0, 0, 0, 0];

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

pub fn generateUUID() -> UID
{
	let mut rng = rand::thread_rng();
	[rng.gen(), rng.gen(), rng.gen(), rng.gen()]
}