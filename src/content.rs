// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::io::{Read, Write, Result};
use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard};

use backend::Backend;
use hash::{HasherFactory, Hasher32, Hash32};

pub trait Content where Self: Sized {
	fn to_content(&self, sink: &mut Sink) -> Result<()>;
	fn from_content(source: &mut Source) -> Result<Self>;
}

/// Implements `Read` + carries along context for constructing types
/// that needs awareness of the backend. `Lazy<T>` for example.
pub struct Source<'a> {
	read: &'a mut Read,
	pub backend: RwLockReadGuard<'a, Box<Backend>>,
	pub backend_arc: &'a Arc<RwLock<Box<Backend>>>,
	pub hasher: &'a HasherFactory,
}

impl<'a> Source<'a> {
	pub fn new(
		read: &'a mut Read,
		hasher: &'a HasherFactory,
		backend_arc: &'a Arc<RwLock<Box<Backend>>>
	) -> Self {
		Source {
			read: read,
			hasher: hasher,
			backend: backend_arc.read(),
			backend_arc: backend_arc,
		}
	}
}

impl<'a> Read for Source<'a> {
   fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
	   self.read.read(buf)
   }
}

pub struct Sink<'a> {
	write: &'a mut Write,
	pub backend: &'a mut Backend,
	hasher: Box<Hasher32>,
}

impl<'a> Sink<'a> {
	pub fn new(
		write: &'a mut Write,
		backend: &'a mut Backend,
		hasher: Box<Hasher32>,
	) -> Self {
		Sink {
			write: write,
			backend: backend,
			hasher: hasher,
		}
	}
	pub fn fin(&mut self) -> Hash32 {
		self.hasher.finalize()
	}
}

impl<'a> Write for Sink<'a> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		try!(self.hasher.write(buf));
		self.write.write(buf)
	}
	fn flush(&mut self) -> Result<()> {
		self.write.flush()
	}
}
