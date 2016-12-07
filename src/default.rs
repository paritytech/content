use std::io::{Write, Result, Error, ErrorKind};
use std::collections::HashMap;
use std::sync::Arc;

use blake2_rfc::blake2b::Blake2b;

pub struct BlakeWrap(Option<Blake2b>);

use hash::{Hash32, Hasher32, HasherFactory};
use backend::Backend;

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

pub fn backend() -> Box<Backend> {
	Box::new(HashMap::<Hash32, Vec<u8>>::new())
}

pub fn hasher() -> HasherFactory {
	Arc::new(|| Box::new(BlakeWrap(Some(Blake2b::new(32)))))
}
