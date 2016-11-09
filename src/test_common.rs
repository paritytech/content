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

extern crate blake2_rfc as b2;
extern crate tempdir;

use tempdir::TempDir;

use std::io::{Read, Write, Result, Error, ErrorKind};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use self::b2::blake2b::Blake2b;

use hash::{Hash32, Hasher32, HasherFactory};
use backend::Backend;
use store::Store;
use content::Content;

pub struct BlakeWrap(Option<Blake2b>);

impl Hasher32 for BlakeWrap {
	fn finalize(&mut self) -> Hash32 {
		let r = self.0.take().expect("Hasher finalized only once").finalize();
		let s = r.as_bytes();
		let mut arr = [0u8; 32];
		for i in 0..32 {
			arr[i] = s[i]
		}
		Hash32(arr)
	}
}

impl Write for BlakeWrap {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let msg = "Write to finished hasher";
		let mut res = Err(Error::new(ErrorKind::Other, msg));
		self.0.iter_mut().next().map(|hasher| res = hasher.write(buf));
		res
	}
	fn flush(&mut self) -> Result<()> {
		let msg = "Flushing finished hasher";
		let mut res = Err(Error::new(ErrorKind::Other, msg));
		self.0.iter_mut().next().map(|hasher| res = hasher.flush());
		res
	}
}

impl Backend for TempDir {
	fn store(
		&mut self,
		source: &Fn(&mut Write, &mut Backend) -> Result<()>,
		hasher: &HasherFactory,
	) -> Result<Hash32> {
		PathBuf::from(self.path()).store(source, hasher)
	}
	fn request(
		&self,
		hash: &Hash32,
		read: &Fn(&mut Read) -> Result<()>,
	) -> Result<()> {
		PathBuf::from(self.path()).request(hash, read)
	}
}

pub fn membackend() -> Box<Backend> {
	Box::new(HashMap::<Hash32, Vec<u8>>::new())
}

pub fn diskbackend() -> Box<Backend> {
	Box::new(TempDir::new("content_pathbuf").unwrap())
}

pub fn store<T: Content>() -> Store<T> {
	Store::new(membackend(), Arc::new(|| {
		Box::new(BlakeWrap(Some(Blake2b::new(32))))
	}))
}

pub fn diskstore<T: Content>() -> Store<T> {
	Store::new(diskbackend(), Arc::new(|| {
		Box::new(BlakeWrap(Some(Blake2b::new(32))))
	}))
}
