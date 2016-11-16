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

use std::io::{Result, Read, Write};

use backend::Backend;
use hash::Hash32;

struct VoidWrite;

impl Write for VoidWrite {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		Ok(buf.len())
	}
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

pub struct VoidBackend;

impl Backend for VoidBackend
{
	fn store(
		&mut self,
		source: &Fn(&mut Write, &mut Backend) -> Result<Hash32>,
	) -> Result<Hash32> {
		source(&mut VoidWrite, self)
	}
	fn request(&self, _: &Hash32, _: &Fn(&mut Read) -> Result<()>,
	) -> Result<()> {
		panic!("attempted to read from VoidBackend")
	}
}
