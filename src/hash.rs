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
use std::io::{Result, Write, Read};
use std::sync::Arc;
use std::fmt;

use byteorder::{BigEndian, ReadBytesExt};

use content::{Content, Source, Sink};

pub type HasherFactory = Arc<Fn() -> Box<Hasher32>>;

impl Hash for Hash32 {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.as_ref().read_u64::<BigEndian>()
			.expect("read from [u8; 32] - don't panic!")
			.hash(state);
	}
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for b in self.0.iter() {
			try!(write!(f, "{:02x}", b))
		}
		Ok(())
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

impl Content for Hash32 {
	fn to_content(&self, sink: &mut Sink) -> Result<()> {
		let res = sink.write_all(&self.0[..]);
		res
	}
	fn from_content(source: &mut Source) -> Result<Self> {
		let mut hash = [0; 32];
		try!(source.read_exact(&mut hash));
		Ok(Hash32(hash))
	}
}
