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

pub mod hashmap;
pub mod void;
pub mod pathbuf;

use std::io::{Read, Write, Result};
use hash::{Hash32, HasherFactory};
pub struct VoidBackend;

pub trait Backend {
	fn store(
		&mut self,
		source: &Fn(&mut Write, &mut Backend) -> Result<()>,
		hasher: &HasherFactory,
	) -> Result<Hash32>;
	fn request(
		&self,
		hash: &Hash32,
		read: &Fn(&mut Read) -> Result<()>,
	) -> Result<()>;
}
