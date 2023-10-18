pub static MAGIC: &'static [u8] = &[0x7f, 0x70, 0x7f, 0x71, 0x7f, 0x72, 0x7f, 0x73];

pub fn find_magic(bytes: &[u8]) -> Option<usize> {
    let mut magic_index = 0;

    for (i, byte) in bytes.iter().enumerate() {
        if *byte == MAGIC[magic_index] {
            magic_index += 1;
            if magic_index == 8 {
                return Some(i - 7);
            }
        } else {
            magic_index = 0;
        }
    }

    None
}
