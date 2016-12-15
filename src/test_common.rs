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

#[cfg(test)]
extern crate tempdir;
use tempdir::TempDir;

use std::io::{Read, Write, Result};
use std::path::PathBuf;

use hash::ContentHasher;
use backend::Backend;
use store::Store;
use content::Content;
use default::BlakeWrap;

impl<H> Backend<H> for TempDir where H: ContentHasher {
	fn store(
		&mut self,
		source: &Fn(&mut Write, &mut Backend<H>) -> Result<H::Digest>,
	) -> Result<H::Digest> {
		PathBuf::from(self.path()).store(source)
	}
	fn request(
		&self,
		hash: &H::Digest,
		read: &Fn(&mut Read) -> Result<()>,
	) -> Result<()> {
		let pb = PathBuf::from(self.path());
		(&pb as &Backend<H>).request(hash, read)
	}
}

pub fn store<T: Content<BlakeWrap>>() -> Store<T, BlakeWrap> {
	Store::new()
}
