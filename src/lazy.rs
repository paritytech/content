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

use content::Content;
use backend::{Backend, VoidBackend};
use hash::{Hash32, NewHash};
use parking_lot::RwLock;
use std::io::{Read, Write, Result};
use std::fmt;

#[derive(Debug)]
struct State<T> {
	value: Option<T>,
	hash: Option<Hash32>,
	written: bool,
}

/// Wraps a value into a lazy content-addressed reference.
///
/// We want to convert Bare --> Hashed --> Written <-> Stored
/// even by &self, so we wrap it in a good ol' RwLock.
pub struct Lazy<T> where T: Content {
	state: RwLock<State<T>>,
	newhash: NewHash,
}

impl<T> PartialEq for Lazy<T> where T: Content {
	fn eq(&self, other: &Self) -> bool {
		self.hashed() == other.hashed()
	}
}

impl<T> Lazy<T> where T: Content {
	pub fn new(inner: T, newhash: NewHash) -> Self {
		Lazy {
			state: RwLock::new(State {
				value: Some(inner),
				hash: None,
				written: false,
			}),
			newhash: newhash,
		}
	}
	fn hashed(&self) -> Hash32 {
		let hash = self.state.read().hash.clone();
		if let Some(hash) = hash {
			hash
		} else {
			let mut backend = VoidBackend;
			let mut lock = self.state.write();
			let t = lock.value
				.take()
				.expect("No hash and no value unreachable");

			let hash = backend.store(&|sink, backend| {
				t.to_content(sink, backend)
			}, &self.newhash).expect("VoidBackend cannot fail");

			lock.hash = Some(hash.clone());
			lock.value = Some(t);
			hash
		}
	}
}

impl<T> Content for Lazy<T> where T: Content {
	fn from_content(
		from: &mut Read,
		newhash: &NewHash,
	) -> Result<Self> {
		let mut hash = [0; 32];
		try!(from.read_exact(&mut hash));
		Ok(Lazy {
			state: RwLock::new(State {
				value: None,
				hash: Some(hash.into()),
				written: true,
			}),
			newhash: newhash.clone(),
		})
	}
	fn to_content(
		&self,
		to: &mut Write,
		backend: &mut Backend,
	) -> Result<()> {
		let new_hash;
		match *self.state.read() {
			State { hash: Some(ref hash), written: true, .. } => {
				// T is already hashed and the wrapped value written
				// just write out the hash, we're done!
				return to.write_all(&hash.0[..]);
			}
			// FIXME, don't hash twice!
			// State { hash: Some(ref hash), written: false, value: ref t } => {

			//	 to.write_all(&hash.0)
			// }
			State { hash: None, value: Some(ref t), .. } => {
				// T neither hashed nor written.
				let hash = try!(backend.store(&|sink, backend| {
					t.to_content(sink, backend)
				}, &self.newhash));
				try!(to.write_all(&hash.0));
				new_hash = Some(hash);
			}
			_ => unreachable!()
		}
		let mut state = self.state.write();
		state.written = true;
		state.hash = new_hash;
		Ok(())
	}
}

impl<T> fmt::Debug for Lazy<T> where T: Content + fmt::Debug {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Lazy {:?}", self.state)
	}
}

#[cfg(test)]
mod tests {
	use test_common;
	use super::Lazy;
	use content::Content;
	use std::io::{Read, Write, Result};
	use hash::NewHash;
	use backend::Backend;

	#[test]
	fn lazy_byte() {
		let mut store = test_common::store();

		let lazy = store.lazy(8u8);
		let hash = store.put(&lazy).unwrap();
		let restored = store.get(&hash).unwrap();

		assert!(lazy == restored);
	}

	#[test]
	fn equality() {
		let mut store = test_common::store();

		let lazy = store.lazy(8u8);
		let unstored = store.lazy(8u8);

		let hash = store.put(&lazy).unwrap();
		let restored = store.get(&hash).unwrap();

		assert!(lazy == unstored);
		assert!(lazy == restored);
	}

	#[derive(PartialEq, Debug)]
	struct LazyFoo {
		a: u8,
		b: Lazy<u8>,
		c: Lazy<Lazy<u8>>,
	}

	impl Content for LazyFoo {
		fn from_content(source: &mut Read, newhash: &NewHash) -> Result<Self> {
			Ok(LazyFoo {
				a: try!(u8::from_content(source, newhash)),
				b: try!(Lazy::from_content(source, newhash)),
				c: try!(Lazy::from_content(source, newhash)),
			})
		}
		fn to_content(
			&self,
			sink: &mut Write,
			backend: &mut Backend
		) -> Result<()> {
			try!(self.a.to_content(sink, backend));
			try!(self.b.to_content(sink, backend));
			self.c.to_content(sink, backend)
		}
	}

	#[test]
	fn lazyfoo() {
		let mut store = test_common::store();

		let lazy = LazyFoo {
			a: 8u8,
			b: store.lazy(8u8),
			c: store.lazy(store.lazy(8u8)),
		};

		let unstored = LazyFoo {
			a: 8u8,
			b: store.lazy(8u8),
			c: store.lazy(store.lazy(8u8)),
		};

		let hash = store.put(&lazy).unwrap();
		let restored = store.get(&hash).unwrap();

		assert_eq!(lazy, unstored);
		assert_eq!(lazy, restored);
	}
}
