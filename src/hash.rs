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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hash32(pub [u8; 32]);

use std::hash::{Hash, Hasher};
use std::io::{Result, Write};
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt};

pub type NewHash = Arc<Box<Fn() -> Box<Hasher32>>>;

impl Hash for Hash32 {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.as_ref().read_u64::<BigEndian>()
			.expect("read from [u8; 32] - don't panic!")
			.hash(state);
	}
}

impl From<[u8; 32]> for Hash32 {
	fn from(array: [u8; 32]) -> Self {
		Hash32(array)
	}
}

pub trait Hasher32
	where Self: Write  {
	fn finalize(&mut self) -> Hash32;
}

pub struct WriteThroughHasher<'a> {
	write: &'a mut Write,
	hasher: Box<Hasher32>,
}

impl<'a> WriteThroughHasher<'a> {
	pub fn new(write: &'a mut Write, hasher: &NewHash) -> Self {
		WriteThroughHasher {
			write: write,
			hasher: hasher(),
		}
	}
	pub fn finalize(&mut self) -> Hash32 {
		self.hasher.finalize()
	}
}

impl<'a> Write for WriteThroughHasher<'a> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		try!(self.hasher.write(buf));
		self.write.write(buf)
	}
	fn flush(&mut self) -> Result<()> {
		self.write.flush()
	}
}
