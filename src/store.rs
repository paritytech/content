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

use hash::{Hash32, HasherFactory};
use backend::Backend;
use content::{Content, Source, Sink};
use default;

pub struct Store<T> {
	backend: Arc<RwLock<Box<Backend>>>,
	hasher: HasherFactory,
	_p: PhantomData<T>,
}

impl<T> Store<T> where T: Content {
	pub fn new_with_backend_and_hasher(
		backend: Box<Backend>,
		hasher: HasherFactory,
	) -> Self {
		Store {
			backend: Arc::new(RwLock::new(backend)),
			hasher: hasher,
			_p: PhantomData,
		}
	}

	pub fn new_with_backend(backend: Box<Backend>) -> Self {
		Self::new_with_backend_and_hasher(
			backend,
			default::hasher(),
		)
	}

	pub fn new_with_hasher(hasher: HasherFactory) -> Self {
		Self::new_with_backend_and_hasher(
			default::backend(),
			hasher,
		)
	}

	pub fn new() -> Self {
		Self::new_with_backend_and_hasher(
			default::backend(),
			default::hasher(),
		)
	}

	pub fn put(&mut self, t: &T) -> Result<Hash32> {
		self.backend.write().store(
			&|write, backend| {
				let mut sink = Sink::new(write, backend, (self.hasher)());
				try!(t.to_content(&mut sink));
				Ok(sink.fin())
			}
		)
	}

	pub fn get(&mut self, hash: &Hash32) -> Result<T> {
		let msg = "Request closure not called";
		let res = RwLock::new(Err(Error::new(ErrorKind::Other, msg)));
		try!(self.backend.read().request(hash, &|read| {
			let mut source = Source::new(
				read,
				&self.hasher,
				self.backend.read(),
			);
			*res.write() = T::from_content(&mut source);
			Ok(())
		}));
		res.into_inner()
	}
}

#[cfg(test)]
mod tests {

	use super::Store;

	#[test]
	fn put_u8() {
		let mut store = Store::new();

		let hash = store.put(&42).unwrap();
		let hash2 = store.put(&43).unwrap();

		assert_eq!(store.get(&hash).unwrap(), 42);
		assert_eq!(store.get(&hash2).unwrap(), 43);
	}

}
