use std::{path::{Path, PathBuf}, fs};

use crate::{events::{EventEmitter, Signal, C_Param}, component::{UID, Component}};

const EVENT_FILESYSTEM_UPDATE: &str = "FileSystemUpdate\0";

#[derive(Debug)]
struct Device
{
	id: PathBuf,
	path: PathBuf,
	mount: PathBuf,
	depth: usize
}

pub struct Filesystem
{
	root: PathBuf,
	name: PathBuf,
	mounts: Vec<Device>,
	emitter: Option<EventEmitter>
}

impl Component for Filesystem
{
	fn uid(&self) -> UID { UID::default() }

	fn listen(&mut self, emitter: Option<EventEmitter>)
	{
		self.emitter = emitter;
    }
}

impl Filesystem
{
	pub fn new(root_path: &str, name: &str) -> Self
	{
		Self
		{
			root: PathBuf::from(root_path),
			name: rel_path(name),
			mounts: Vec::new(),
			emitter: None
		}
	}

	pub fn root(&self) -> &Path
	{
		self.root.as_path()
	}

	pub fn mount(&mut self, device: &str, mount: &str) -> bool
	{
		let device_path = rel_path(device);
		let mount_path = rel_path(mount);

		if let Some(id) = self.device_path(&device_path)
		{
			for mount in &self.mounts
			{
				if mount.id == id { return false; }
				if mount.mount == mount_path { return false; }
			}
			let path = [&self.root, id].iter().collect::<PathBuf>().canonicalize().unwrap();
			//println!("{:?}", [&self.root, id]);
			let device = Device{
				id: id.to_owned(),
				path: path,
				depth: mount_path.components().count(),
				mount: mount_path.clone()
			};
			self.mounts.push(device);
			// Stelle sicher, dass tiefere Mountpoints vor höheren abgesucht werden.
			self.mounts.sort_by(|a, b| b.depth.cmp(&a.depth));
			self.fire_filesystem_update(0, Some(&mount_path), None);
			true
		}
		else
		{
			false
		}
	}

	pub fn unmount(&mut self, device: &str) -> bool
	{
		let device_path = rel_path(device);
		if let Some(id) = self.device_path(&device_path)
		{
			for i in 0..self.mounts.len()
			{
				if self.mounts[i].id == id
				{
					let d = self.mounts.remove(i);
					self.fire_filesystem_update(0, Some(&d.mount), None);
					return true;
				}
			}
		}
		false
	}

	pub fn exists(&self, path_name: &str) -> bool
	{
		match self.resolve(path_name)
		{
			Some(path) => path.exists(),
			None => false
		}
	}

	pub fn is_file(&self, path_name: &str) -> bool
	{
		match self.resolve(path_name)
		{
			Some(path) => path.is_file(),
			None => false
		}
	}

	pub fn is_dir(&self, path_name: &str) -> bool
	{
		match self.resolve(path_name)
		{
			Some(path) => path.is_dir(),
			None => false
		}
	}

	pub fn remove(&self, path_name: &str) -> bool
	{
		match self.resolve(path_name)
		{
			Some(path) => if path.is_file()
			{
				fs::remove_file(path).is_ok()
			}
			else
			{
				fs::remove_dir(path).is_ok()
			},
			None => false
		}
	}

	pub fn create_dir(&self, path_name: &str) -> bool
	{
		match self.resolve(path_name)
		{
			Some(path) => fs::create_dir_all(path).is_ok(),
			None => false
		}
	}

	pub fn rename(&self, from: &str, to: &str) -> bool
	{
		if let Some(from_path) = self.resolve(from)
		{
			match self.resolve(to)
			{
				Some(to_path) => return fs::rename(from_path, to_path).is_ok(),
				None => {}
			}
		}
		false
	}

	pub fn childs(&self, path_name: &str) -> Vec<String>
	{
		let mut items = Vec::<String>::new();
		match self.resolve(path_name)
		{
			Some(path) => {
				for item in fs::read_dir(path).expect("Error while reading directory.")
				{
					let entry = item.expect("Error while reading file.");
					println!("resolved {:?}", entry);
					let file_path = path_name.to_owned() + "/" + entry.file_name().to_str().unwrap() + "\0";
					items.push(file_path);
				}
			},
			None => {}
		}
		items
	}

	pub fn real_path(&self, path_name: &str) -> Option<String>
	{
		let path = self.resolve(path_name)?;
		let str = path.to_str()?;
		Some(str.to_owned())
	}

	fn resolve(&self, path_name: &str) -> Option<PathBuf>
	{
		let path = rel_path(path_name);
		for mount in &self.mounts
		{
			if let Some(rel_path) = path.strip_prefix(&mount.mount).ok()
			{
				let full_path: PathBuf = [&mount.path, &PathBuf::from(rel_path)].iter().collect();
				// Teste, ob wir noch innerhalb des Dateisystems sind.
				if !full_path.starts_with(&mount.path) { return None }
				//println!("resolved {:?}", full_path);
				return Some(full_path);
			}
		}
		None
	}

	fn device_path<'a>(&self, path: &'a PathBuf) -> Option<&'a Path>
	{
		path.strip_prefix(&self.name).ok()
	}

	fn fire_filesystem_update(&mut self, event_type: u32, path: Option<&PathBuf>, new_path: Option<&PathBuf>)
	{
		if let Some(e) = self.emitter.as_mut()
		{
			let mut params = Vec::<C_Param>::new();
			params.push(C_Param::from(&event_type, true));
			if let Some(p) = path { params.push(c_param(p)); }
			if let Some(p) = new_path { params.push(c_param(p)); }

			e.send(Signal::raw(EVENT_FILESYSTEM_UPDATE, e.owner(), params));
		}
	}
}

#[inline]
fn rel_path(str: &str) -> PathBuf
{
	PathBuf::from(str.trim_start_matches("/"))
}

#[inline]
fn c_param(path: &PathBuf) -> C_Param
{
	C_Param::from_str(&path.to_str().unwrap())
}