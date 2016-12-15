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

use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

use content::{Content, Sink, Source};

pub trait ContentHasher where Self: Write + Sized {
	type Digest: Eq + Hash + Clone + Content<Self> + AsRef<[u8]>;

	fn new() -> Self;
	fn fin(self) -> Self::Digest;
}

struct VoidHasher;

#[derive(PartialEq, Eq, Clone)]
struct VoidHash;

impl ContentHasher for VoidHasher {
	type Digest = VoidHash;

	fn new() -> Self {
		VoidHasher
	}

	fn fin(self) -> Self::Digest {
		VoidHash
	}
}

impl AsRef<[u8]> for VoidHash {
	fn as_ref(&self) -> &[u8] {
		&[]
	}
}

impl Hash for VoidHash {
	fn hash<H>(&self, _: &mut H) where H: Hasher {}
}

impl<H> Content<H> for VoidHash where H: ContentHasher {
	fn to_content(&self, _: &mut Sink<H>) -> Result<()> {
		Ok(())
	}
	fn from_content(_: &mut Source<H>) -> Result<Self> {
		Ok(VoidHash)
	}
}

impl Write for VoidHasher {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		Ok(buf.len())
	}
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}
