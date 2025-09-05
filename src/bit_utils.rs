pub fn carry_check_add_8bit(a: u8, b: u8) -> bool {
    (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
}

pub fn carry_check_sub_8bit(a: u8, b: u8) -> bool {
    (((a & 0xF) - (b & 0xF)) & 0x10) == 0x10
}

pub fn concatenate_bytes(lower_byte: u8, higher_byte: u8) -> u16 {
    lower_byte as u16 | ((higher_byte as u16) << 8)
}

pub fn split_2bytes(bytes: u16) -> (u8, u8) {
    let high_byte = (bytes >> 8) as u8;
    let low_byte = (bytes & 0xFF) as u8;
    (low_byte, high_byte)
}