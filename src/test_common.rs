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

extern crate tempdir;

use tempdir::TempDir;

use std::io::{Read, Write, Result};
use std::path::PathBuf;

use hash::Hash32;
use backend::Backend;
use store::Store;
use content::Content;

impl Backend for TempDir {
	fn store(
		&mut self,
		source: &Fn(&mut Write, &mut Backend) -> Result<Hash32>,
	) -> Result<Hash32> {
		PathBuf::from(self.path()).store(source)
	}
	fn request(
		&self,
		hash: &Hash32,
		read: &Fn(&mut Read) -> Result<()>,
	) -> Result<()> {
		PathBuf::from(self.path()).request(hash, read)
	}
}

pub fn tempdisk() -> Box<Backend> {
	Box::new(TempDir::new("content_pathbuf").unwrap())
}

pub fn diskstore<T: Content>() -> Store<T> {
	Store::new_with_backend(tempdisk())
}
