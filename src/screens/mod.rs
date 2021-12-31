use std::sync::{Arc, Mutex};

use self::screen::ScreenComponent;


pub mod renderer;
pub mod screen;
mod grid;
mod text;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color
{
	r: f32,
	g: f32,
	b: f32,
	a: f32
}

impl Color
{
	pub const BLACK: Self = Self {r: 0.0,g: 0.0, b: 0.0, a: 1.0};
    pub const WHITE: Self = Self {r: 1.0,g: 1.0, b: 1.0, a: 1.0};

	pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color
	{
		Color{r, g, b, a}
	}

	pub fn learp(&self, other: Color, v: f32) -> Color
	{
		let r2 = self.r;
		let g2 = self.g;
		let b2 = self.b;
		let a2 = self.a;
		let r = (other.r - r2) * v + r2;
		let g = (other.g - g2) * v + g2;
		let b = (other.b - b2) * v + b2;
		let a = (other.a - a2) * v + a2;
		Color{r, g, b, a}
	}

	#[inline]
	pub fn bytes(&self) -> [u8; 4]
	{
		[(self.r * 255.0) as u8, (self.g * 255.0) as u8, (self.b * 255.0) as u8, (self.a * 255.0) as u8]
	}

	/*pub fn hsl(&self) -> (f32, f32, f32)
	{
		let r = self.red() as f32 / 255.;
		let g = self.green() as f32 / 255.;
		let b = self.blue() as f32 / 255.;
		let max = max!(r, g, b);
		let min = min!(r, g, b);
		let mut h: f32;
		let s: f32;
		let l: f32 = (max + min) / 2.;
	
		if (max == min)
		{
			h = 0.;
			s = 0.;
		}
		else
		{
			let d = max - min;
			s = if (l > 0.5) { d / (2.0 - max - min) } else { d / (max + min) };
			h = if (max == r)
			{
				(g - b) / d + if (g < b) {6.} else {0.}
			}
			else if (max == g)
			{
				(b - r) / d + 2.
			}
			else
			{
				(r - g) / d + 4.
			};
			h /= 6.0;
		}
		return (h, s, l);
    }*/
}

impl Into<[f32; 4]> for Color
{
	#[inline]
    fn into(self) -> [f32; 4]
	{
		[self.r, self.g, self.b, self.a]
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Size
{
	pub width: u32,
	pub height: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct ScreenSize
{
	pub grid_width: u32,
	pub grid_height: u32,
	pub window_width: u32,
	pub window_height: u32,
	pub font_size: u32,
}

impl ScreenSize
{
	pub fn from_grid(width: u32, height: u32, font_size: u32) -> Self
	{
		Self
		{
			grid_width: width,
			grid_height: height,
			window_width: width * font_size / 2,
			window_height: height * font_size,
			font_size
		}
	}

	#[inline]
	pub fn cell_size(&self) -> Size
	{
		Size{width: self.font_size / 2, height: self.font_size}
	}

	pub fn resize_grid(&mut self, width: u32, height: u32)
	{
		self.grid_width = width;
		self.grid_height = height;
		self.window_width = width * self.font_size / 2;
		self.window_height = height * self.font_size;
	}
}

#[repr(C)]
pub struct GraphicHandle
{
	pub buffer: Arc<Mutex<Buffer>>,
	pub screen: Option<ScreenComponent>,
	pub fg: Color,
	pub bg: Color
}

impl GraphicHandle
{
	pub fn new(width: u32, height: u32) -> Self
	{
		Self
		{
			buffer: Arc::new( Mutex::new(Buffer::new(width, height))),
			screen: None,
			fg: Color::WHITE,
			bg: Color::BLACK,
		}
	}

	pub fn bind_screen(&mut self, screen: Option<ScreenComponent>)
	{
		self.screen = screen;
		if self.screen.is_some()
		{
			self.screen.as_mut().unwrap().open(self.buffer.clone());
		}
	}

	pub fn resize_screen(&mut self, width: u32, height: u32)
	{
		self.buffer.lock().unwrap().resize(width, height);
	}

	pub fn get_buffer(&self) -> Buffer
	{
		let buffer;
		{
			let b = self.buffer.lock().unwrap();
			//self.buffer.lock().unwrap().clone(
			buffer = b.clone();
		}
		let _ = self.buffer.lock().unwrap();
		buffer
	}

	pub fn set_buffer(&self, other: &Buffer)
	{
		self.buffer.lock().unwrap().replace(other);
	}

	pub fn exec<F>(&self, mut func: F) where F: FnMut(&mut Buffer)
	{
		func(&mut self.buffer.lock().unwrap());
	}

	pub fn flush(&mut self)
	{
		if self.screen.is_some()
		{
			self.screen.as_ref().unwrap().flush();
		}
	}
}

#[derive(Debug, Clone)]
pub struct Buffer
{
	pub width: u32,
	pub height: u32,
	pub chars: Vec<char>,
	pub foreground: Vec<Color>,
	pub background: Vec<Color>,
}

impl Buffer
{
	pub fn new(width: u32, height: u32) -> Buffer
	{
		let size = (width * height) as usize;

		let mut chars = Vec::with_capacity(size);
		chars.resize(size, char::default());

		let mut foreground = Vec::with_capacity(size);
		foreground.resize(size, Color::WHITE);

		let mut background = Vec::with_capacity(size);
		background.resize(size, Color::BLACK);

		Buffer
		{
			width,
			height,
			chars,
			foreground,
			background,
		}
	}

	pub fn resize(&mut self, width: u32, height: u32)
	{
		self.width = width;
		self.height = height;
		let size = (width * height) as usize;

		self.chars.resize(size, char::default());
		self.foreground.resize(size, Color::WHITE);
		self.background.resize(size, Color::BLACK);
	}

	pub fn copy(&mut self, _x: i32, _y: i32, _other: &Buffer)
	{
		/*let x1 = x.max(0) as usize;
		let y1 = y.max(0) as usize;
		let x2: usize;
		if x < 0
			{ x2 = (other.width - (-x) as u32).min(self.width) as usize; }
		else
			{ x2 = (self.width - x as u32).min(other.width) as usize; }
		let y2: usize;
		if y < 0
			{ y2 = (other.height - (-y) as u32).min(self.height) as usize; }
		else
			{ y2 = (self.height - y as u32).min(other.height) as usize; }

		let n = (self.width as usize - x1).min(other.width as usize - x2);
		for line in y1..y2
		{
			let d1 = self.width * line;
			let d2 = other.width * line;

			self.chars[(x1 + d1)..(x1+d1+n)].copy_from_slice(&other.chars[(x2+d2)..(x2+d2+n)]);
		}*/
	}

	pub fn replace(&mut self, other: &Buffer)
	{
		self.width = other.width;
		self.height = other.height;
		self.chars = other.chars.clone();
		self.foreground = other.foreground.clone();
		self.background = other.background.clone();
	}

	pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, char: char, fg: Color, bg: Color)
	{
		for yy in 0..h
		{
			let y1 = yy+y;
			if y1 < 0 { continue; }
			if y1 as u32 >= self.height { break; }
			for xx in 0..w
			{
				let x1 = xx+x;
				if x1 < 0 { continue; }
				if x1 as u32 >= self.width { break; }
				self.write(x1, y1, char, fg, bg);
			}
		}
	}

	pub fn writeText(&mut self, x: i32, y: i32, text: &str, fg: Color, bg: Color)
	{
		for (i, ch) in text.chars().enumerate()
		{
			self.write(x+i as i32, y, ch, fg, bg);
		}

		//println!("Buffer: {},{} -> {}", x, y, text);
	}

	pub fn write(&mut self, x: i32, y: i32, char: char, fg: Color, bg: Color)
	{
		if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 { return; }
		let idx = x as usize + y as usize * self.width as usize;
		if self.chars[idx] == char && self.foreground[idx] == fg && self.background[idx] == bg { return; }
		self.chars[idx] = char;
		self.foreground[idx] = fg;
		self.background[idx] = bg;

		//println!("Buffer Write: {},{}", x, y);
	}

	/*pub fn get_cell(&self, x: u32, y: u32) -> BufferCell
	{
		let idx = x as usize + y as usize * self.width as usize;
		BufferCell
		{
			char: self.chars[idx],
			fg: self.foreground[idx], 
			bg: self.background[idx], 
		}
	}*/
}

#[repr(C)]
pub struct BufferCell
{
	pub char: *mut String,
	pub fg: Color,
	pub bg: Color,
}
