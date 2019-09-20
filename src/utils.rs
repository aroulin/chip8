pub fn make_byte(high: u16, low: u16) -> u8 {
    assert!(high <= 0xFF);
    assert!(low <= 0xFF);
    ((high << 4) | low) as u8
}

pub fn make_tribble(n1: u16, n2: u16, n3: u16) -> u16 {
    assert!(n1 <= 0xFF);
    assert!(n2 <= 0xFF);
    assert!(n3 <= 0xFF);
    ((n1 << 8) | (n2 << 4) | n3) as u16
}
