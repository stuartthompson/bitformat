pub trait ByteSource {
    // Gets the next byte from the byte source.
    pub fn next(): u8;
    // Gets the total number of bytes the source will provide.
    pub fn len(): u64;
}