use std::fmt;

#[derive(Debug)]
pub struct MacAddress(pub [u8; 6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        );
    }
}

#[derive(Debug)]
pub struct IPv4Addr(pub [u8; 4]);

impl fmt::Display for IPv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3]);
    }
}
