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

use std::sync::Arc;
use std::io::{Result, Error, ErrorKind};
use std::marker::PhantomData;

use parking_lot::RwLock;

use hash::{Hash32, Hasher32, NewHash};
use backend::Backend;
use content::Content;
use lazy::Lazy;

pub struct Store<T> {
	backend: Box<Backend>,
	newhash: NewHash,
	_p: PhantomData<T>,
}

impl<T> Store<T> where T: Content {
	pub fn new(
		backend: Box<Backend>,
		factory: Box<Fn() -> Box<Hasher32>>,
	) -> Self {
		Store {
			backend: backend,
			newhash: Arc::new(factory),
			_p: PhantomData,
		}
	}

	pub fn lazy<U: Content>(&self, inner: U) -> Lazy<U> {
		Lazy::new(inner, self.newhash.clone())
	}

	pub fn put(&mut self, t: &T) -> Result<Hash32> {
		self.backend.store(
			&|sink, backend| {
				t.to_content(sink, backend)
			},
			&self.newhash
		)
	}

	pub fn get(&mut self, hash: &Hash32) -> Result<T> {
		let msg = "Request closure not called";
		let res = RwLock::new(Err(Error::new(ErrorKind::Other, msg)));
		try!(self.backend.request(hash, &|read| {
			*res.write() = Ok(try!(T::from_content(read, &self.newhash)));
			Ok(())
		}));
		res.into_inner()
	}
}

#[cfg(test)]
mod tests {

	use test_common;

	#[test]
	fn put_u8() {
		let mut store = test_common::store::<u8>();

		let hash = store.put(&42).unwrap();
		let hash2 = store.put(&43).unwrap();

		assert_eq!(store.get(&hash).unwrap(), 42);
		assert_eq!(store.get(&hash2).unwrap(), 43);
	}

}
