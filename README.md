# Content

Design goal of the Content storage/serialization backend.

## Simple implementation of Content trait for any type
   When implementing Content for your type, you need to provide only two methods, `to_content` that takes a Read, and produces a Self, and `from_content`, an example for u8:
```rust
impl Content for u8 {
    fn to_content(&self, sink: &mut Sink) -> Result<()> {
        sink.write_all(&[*self])
    }
    fn from_content(source: &mut Source) -> Result<Self> {
        let b = &mut [0u8];
        try!(source.read_exact(b));
        Ok(b[0])
    }
}
```
## The `Backend` trait
   The backend trait makes a type act as as storage for content addressed bytestreams, be it `Vec<u8>` or `File`. But it could also be a network interface, for example.

## The type is the schema
   Things that implement Content, implicitly know how to serialize and deserialize themselves, including knowing their length. And since types can nest other Content types within themselves, you basically get the whole schema and de/serialization for free.
## Zero copy io
   When a Content is put into a store, Reads and Writes are connected directly.
## Transparant lazy references
   If you have a type implementing Content that is larger than would make sense to serialize directly, such as a tree structure, you can put the children behind Lazy<T> reference types, which means they will only be lazily loaded off disk and cached as they are needed.
## Hash while writing, only hash once
   Hashing is much more expensive than serialization, therefore we assure that any data is only hashed once
