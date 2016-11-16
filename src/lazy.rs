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

use content::{Content, Source, Sink};
use backend::Backend;
use backend::void::VoidBackend;
use hash::{Hash32, HasherFactory};
use std::sync::Arc;

use std::ops::Deref;
use parking_lot::{RwLock, RwLockReadGuard};
use std::io::{Result, Error, ErrorKind};
use std::fmt;
use std::marker::PhantomData;
use std::cell::UnsafeCell;

pub enum State {
	/// value, !written
	Bare,

	/// value, hashed, !written
	Hashed,

	/// hashed, written
	Lazy,

	/// value, hashed, written
	Full,
}

/// Wraps a value into a lazy content-addressed reference.
pub struct Lazy<T> where T: Content {
	state: RwLock<State>,

	value: UnsafeCell<Option<T>>,
	hash: UnsafeCell<Option<Hash32>>,

	backend: Arc<RwLock<Box<Backend>>>,
	hasher: HasherFactory,
}

pub struct LazyRef<'a, T: 'a> {
	_guard: RwLockReadGuard<'a, State>,
	value: &'a T,
	_marker: PhantomData<&'a T>,
}

impl<'a, T> LazyRef<'a, T> {
	pub fn new(guard: RwLockReadGuard<'a, State>, value: &'a T) -> Self {
		LazyRef {
			_guard: guard,
			value: value,
			_marker: PhantomData,
		}
	}
}

impl<'a, T> Deref for LazyRef<'a, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.value
	}
}

impl<T> Lazy<T> where T: Content {
	pub fn new(
		inner: T,
		hasher: HasherFactory,
		backend: Arc<RwLock<Box<Backend>>>,
	) -> Self {
		Lazy {
			state: RwLock::new(State::Bare),
			value: UnsafeCell::new(Some(inner)),
			hash: UnsafeCell::new(None),

			hasher: hasher.clone(),
			backend: backend,
		}
	}
	pub fn hash(&self) -> LazyRef<Hash32> {
		let readlock = self.state.read();
		match *readlock {
			State::Hashed
			| State::Lazy
            | State:: Full => unsafe {
				LazyRef::new(
					readlock,
					&(*self.hash.get())
						.as_ref()
						.expect("These states are hashed"))
			},
			State::Bare => {
				let mut backend = VoidBackend;
				unsafe {
					*self.hash.get() = Some(backend.store(&|write, backend| {
						let t = &(*self.value.get())
							.as_ref()
							.expect("Bare always has value");
						let mut sink = Sink::new(
							write,
							backend,
							(self.hasher)()
						);
						try!(t.to_content(&mut sink));
						Ok(sink.fin())
					}).expect("VoidBackend cannot fail"));
					LazyRef::new(
						readlock,
						&(*self.hash.get())
							.as_ref()
							.expect("These states are hashed"))
				}
			}
		}
	}
	pub fn as_ref<'a>(&'a self) -> Result<LazyRef<T>> {
		{
			let readlock = self.state.read();
			match *readlock {
				State::Bare
					| State::Hashed
					| State:: Full => unsafe {
						return Ok(LazyRef::new(
							readlock,
							&(*self.value.get())
								.as_ref()
								.expect("These states have value")))
					},
				_ => (),
			}
		}
		// Only State::Lazy past this point
		{
			// Get a writelock to change the State of Lazy
			let mut writelock = self.state.write();
			let hash = unsafe {
				&(*self.hash.get())
					.as_ref()
					.expect("These states have value")
			};
			let msg = "Request closure not called";
			let res = RwLock::new(Err(Error::new(ErrorKind::Other, msg)));
			try!(self.backend.read().request(hash, &|read| {
				let mut source = Source::new(
					read,
					&self.hasher,
					&self.backend
				);
				*res.write() = T::from_content(&mut source);
				Ok(())
			}));
			unsafe { *self.value.get() = Some(try!(res.into_inner())); };
			*writelock = State::Full;
		}
		Ok(LazyRef::new(
			self.state.read(),
			unsafe { &(*self.value.get())
						.as_ref()
						.expect("These states have value")
			}))
	}
}

impl<T> Content for Lazy<T> where T: Content {
	fn from_content(source: &mut Source) -> Result<Self> {
		Ok(Lazy {
			state: RwLock::new(State::Lazy),
			value: UnsafeCell::new(None),
			hash: UnsafeCell::new(Some(try!(Hash32::from_content(source)))),

			hasher: source.hasher.clone(),
			backend: source.backend_arc.clone(),
		})
	}
	fn to_content(&self, sink: &mut Sink) -> Result<()> {
		match *self.state.read() {
			State::Lazy | State:: Full => {
				let hash = unsafe { &*self.hash.get() };
				return hash.to_content(sink);
			}
			_ => () // go on
		}
		let mut writeguard = self.state.write();
		match *writeguard {
			State::Bare | State::Hashed => {
				// TODO, don't re-hash
				unsafe {
					let hash = try!(sink.backend.store(&|write, backend| {
						let t = &(*self.value.get())
							.as_ref()
							.expect("Bare and Hashed always has value");
						let mut inner_sink = Sink::new(
							write,
							backend,
							(self.hasher)(),
						);
						try!(t.to_content(&mut inner_sink));
						Ok(inner_sink.fin())
					}));
					try!(hash.to_content(sink));
					*self.hash.get() = Some(hash);
				}
				*writeguard = State::Full;
				Ok(())
			},
			_ => unreachable!(),
		}
	}
}

impl<T> fmt::Debug for Lazy<T> where T: Content + fmt::Debug {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self.state.read() {
			State::Bare =>
				write!(f, "Bare {:?}", &self.value),
			State::Hashed =>
				write!(f, "Hashed {:?} {:?}", &self.value, &self.hash),
			State::Lazy =>
				write!(f, "Lazy {:?}", &self.hash),
			State::Full =>
				write!(f, "Full {:?} {:?}", &self.value, &self.hash),
		}
	}
}

impl<T> PartialEq for Lazy<T> where T: Content {
	fn eq(&self, other: &Self) -> bool {
		*self.hash() == *other.hash()
	}
}

#[cfg(test)]
mod tests {
	use test_common;
	use super::Lazy;
	use content::{Content, Source, Sink};
	use std::io::Result;

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
		fn from_content(source: &mut Source) -> Result<Self> {
			Ok(LazyFoo {
				a: try!(u8::from_content(source)),
				b: try!(Lazy::from_content(source)),
				c: try!(Lazy::from_content(source)),
			})
		}
		fn to_content(&self, sink: &mut Sink) -> Result<()> {
			try!(self.a.to_content(sink));
			try!(self.b.to_content(sink));
			self.c.to_content(sink)
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

	#[test]
	fn lazy_deref() {
		let mut store = test_common::store::<LazyFoo>();

		let foo = LazyFoo {
			a: 8u8,
			b: store.lazy(8u8),
			c: store.lazy(store.lazy(8u8)),
		};

		{
			assert_eq!(*foo.b.as_ref().unwrap(), 8u8);
			let inner = foo.c.as_ref().unwrap();
			assert_eq!(*inner.as_ref().unwrap(), 8u8);
		}

		let hash = store.put(&foo).unwrap();
		let restored = store.get(&hash).unwrap();

		assert_eq!(*restored.b.as_ref().unwrap(), 8u8);
		let inner = restored.c.as_ref().unwrap();
		assert_eq!(*inner.as_ref().unwrap(), 8u8);
	}
}
