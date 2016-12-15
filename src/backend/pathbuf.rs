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

use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Result, Read, Write};
use std::fmt::Write as FmtWrite;
use rand::{thread_rng, Rng};

use backend::Backend;
use hash::ContentHasher;

fn pathbuf_from_hash<H: ContentHasher>(
	root: &PathBuf,
	hash: &H::Digest
) -> Result<PathBuf> {
	let bytes: &[u8] = hash.as_ref();
	let a = format!("{:02x}", bytes[0]);
	let b = format!("{:02x}", bytes[1]);
	let mut rest = String::new();
	for i in 2..32 {
		write!(&mut rest, "{:02x}", bytes[i])
			.expect("in-memory write to succeed");
	};
	let mut pathbuf = root.clone();

	pathbuf.push(a);
	pathbuf.push(b);
	try!(fs::create_dir_all(pathbuf.to_str().unwrap()));
	pathbuf.push(rest);
	Ok(pathbuf)
}

fn temporary_path(root: &PathBuf) -> PathBuf {
	let mut rand = root.clone();
	let s: String = thread_rng().gen_ascii_chars().take(10).collect();
	rand.push(s);
	rand
}

impl<H> Backend<H> for PathBuf where H: ContentHasher {
	fn store(
		&mut self,
		source: &Fn(&mut Write, &mut Backend<H>) -> Result<H::Digest>
	) -> Result<H::Digest> {
		let tmp_path = temporary_path(self);
		let mut file = try!(File::create(&tmp_path));
		let hash = try!(source(&mut file, self));
		let pathbuf = try!(pathbuf_from_hash::<H>(self, &hash));
		try!(fs::rename(tmp_path, pathbuf));
		Ok(hash)
	}
	fn request(
		&self,
		hash: &H::Digest,
		read: &Fn(&mut Read) -> Result<()>,
	) -> Result<()> {
		let pathbuf = try!(pathbuf_from_hash::<H>(self, &hash));
		let mut file = try!(File::open(pathbuf));
		read(&mut file)
	}
}

#[cfg(test)]
mod tests {
	extern crate tempdir;
	use store::Store;
	use tempdir::TempDir;
	use default::BlakeWrap;

	#[test]
	fn disk_write_get() {
		let mut store: Store<_, BlakeWrap> = Store::new_with_backend(
			Box::new(TempDir::new("content").unwrap())
		);
		let thing = Some(Box::new(8u8));
		let hash = store.put(&thing).unwrap();
		let retrieved = store.get(&hash).unwrap();
		assert_eq!(thing, retrieved);
	}
}
