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

use parking_lot::RwLockReadGuard;

use backend::Backend;
use hash::ContentHasher;

pub trait Content<H> where Self: Sized, H: ContentHasher {
	fn to_content(&self, sink: &mut Sink<H>) -> Result<()>;
	fn from_content(source: &mut Source<H>) -> Result<Self>;
}

pub struct Source<'a, H> where H: 'a + ContentHasher {
	read: &'a mut Read,
	pub backend: RwLockReadGuard<'a, Box<Backend<H>>>,
}

impl<'a, H> Source<'a, H> where H: ContentHasher {
	pub fn new(
		read: &'a mut Read,
		backend: RwLockReadGuard<'a, Box<Backend<H>>>,
	) -> Self {
		Source {
			read: read,
			backend: backend,
		}
	}
}

impl<'a, H> Read for Source<'a, H> where H: ContentHasher {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		self.read.read(buf)
	}
}

pub struct Sink<'a, H> where H: 'a {
	write: &'a mut Write,
	pub backend: &'a mut Backend<H>,
	hasher: H,
}

impl<'a, H> Sink<'a, H> where H: ContentHasher {
	pub fn new(
		write: &'a mut Write,
		backend: &'a mut Backend<H>,
	) -> Self {
		Sink {
			write: write,
			backend: backend,
			hasher: H::new(),
		}
	}
	pub fn fin(self) -> H::Digest {
		self.hasher.fin()
	}
}

impl<'a, H> Write for Sink<'a, H> where H: ContentHasher {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		try!(self.hasher.write(buf));
		self.write.write(buf)
	}
	fn flush(&mut self) -> Result<()> {
		self.write.flush()
	}
}
