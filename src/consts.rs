//
// Block sizes are 2048 bytes
pub const CISO_BLOCK_SIZE: u32 = 0x800;

// Headers are 24 bytes
pub const CISO_HEADER_SIZE: u32 = 0x18;

// File header magic, "CISO" in ASCII, little endian.
pub const CISO_MAGIC: u32 = 0x4F534943;

// Window size for compression is 15, with no compression header.
pub const CISO_WINDOW_SIZE: u8 = 15;

