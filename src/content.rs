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
use backend::void::VoidBackend;
use hash::{self, HasherFactory, Hasher32, Hash32};

pub trait Content where Self: Sized {
	fn to_content(&self, sink: &mut Sink) -> Result<()>;
	fn from_content(source: &mut Source) -> Result<Self>;

	fn content_len(&self) -> usize {
		let mut write = CountingWrite(0);
		{
			let mut backend = VoidBackend;
			let mut sink = Sink::new(&mut write,
									 &mut backend,
									 hash::voidhasher());
			self.to_content(&mut sink).expect("Cannot fail");
		}
		write.written_len()
	}
}

pub struct CountingWrite(usize);

impl CountingWrite {
	fn written_len(&self) -> usize {
		self.0
	}
}

impl Write for CountingWrite {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let len = buf.len();
		self.0 += len;
		Ok(len)
	}
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

pub struct Source<'a> {
	read: &'a mut Read,
	pub backend: RwLockReadGuard<'a, Box<Backend>>,
	pub hasher: &'a HasherFactory,
}

impl<'a> Source<'a> {
	pub fn new(
		read: &'a mut Read,
		hasher: &'a HasherFactory,
		backend: RwLockReadGuard<'a, Box<Backend>>,
	) -> Self {
		Source {
			read: read,
			hasher: hasher,
			backend: backend,
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

#[cfg(test)]
mod tests {
    use content::Content;

	#[test]
	fn content_len() {
		assert_eq!(8u8.content_len(), 1);
		assert_eq!(Some(8u8).content_len(), 2);
		assert_eq!(8u64.content_len(), 8);
	}
}
