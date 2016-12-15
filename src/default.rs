use std::io::{Read, Write, Result};
use std::hash::{Hash, Hasher};
use std::mem;

use blake2_rfc::blake2b::Blake2b;
use byteorder::{ReadBytesExt, BigEndian};
use content::{Content, Source, Sink};

pub struct BlakeWrap(Blake2b);

#[derive(PartialEq, Eq, Clone)]
pub struct BlakeResultWrap([u8; 32]);

use hash::ContentHasher;

impl ContentHasher for BlakeWrap {
	type Digest = BlakeResultWrap;

	fn new() -> Self {
		BlakeWrap(Blake2b::new(32))
	}

	fn fin(self) -> Self::Digest {
		let mut bytes: [u8; 32];
		unsafe { bytes = mem::uninitialized() }
		let fin = self.0.finalize();
		let resultbytes = fin.as_bytes();
		for i in 0..32 {
			bytes[i] = resultbytes[i]
		}
		BlakeResultWrap(bytes)
	}
}

impl AsRef<[u8]> for BlakeResultWrap {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl Hash for BlakeResultWrap {
	fn hash<H>(&self, state: &mut H) where H: Hasher {
		state.write_u64((&self.0[..])
						.read_u64::<BigEndian>()
						.expect("digest than 8 bytes"))
	}
}

impl<H> Content<H> for BlakeResultWrap where H: ContentHasher {
	fn to_content(&self, sink: &mut Sink<H>) -> Result<()> {
		sink.write_all(&self.0)
	}
	fn from_content(source: &mut Source<H>) -> Result<Self> {
		let mut bytes: [u8; 32];
		unsafe { bytes = mem::uninitialized() }
		try!(source.read_exact(&mut bytes[..]));
		Ok(BlakeResultWrap(bytes))
	}
}

impl Write for BlakeWrap {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.0.write(buf)
	}
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use store::Store;
	use default::BlakeWrap;

	#[test]
	fn hash_of_hash() {
		let mut store1: Store<_, BlakeWrap> = Store::new();
		let mut store2: Store<_, BlakeWrap> = Store::new();

		let val = 42u8;
		let hash1 = store1.put(&val).unwrap();
		let hash2 = store2.put(&hash1).unwrap();

		let hash1_b = store2.get(&hash2).unwrap();
		let val_b = store1.get(&hash1_b).unwrap();

		assert!(hash1 == hash1_b);
		assert!(val == val_b);
	}
}
