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

use backend::{Backend, VoidBackend};
use hash::{Hash32, NewHash};

impl Backend for VoidBackend
{
	fn store(
		&mut self,
		source: &Fn(&mut Write, &mut Backend) -> Result<()>,
		hasher: &NewHash,
	) -> Result<Hash32> {
		let mut h = hasher();
		try!(source(&mut h, self));
		Ok(h.finalize())
	}
	fn request(&self, _: &Hash32, _: &Fn(&mut Read) -> Result<()>,
	) -> Result<()> {
		panic!("attempted to read from VoidBackend")
	}
}
