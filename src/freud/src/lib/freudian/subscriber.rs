use libq::rand::fill_exact;
use libq::blkid::UUID;
use std::io;

pub struct Subscription {
    pub id: String,
    pub offset: u64
}

impl Subscription {
    pub fn new() -> Result<Subscription, io::Error> {
        let mut buffer = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        fill_exact(&mut buffer)?;

        return Ok(Subscription{
            id: UUID::from_slice16(buffer).to_string(),
            offset: 0
        });
    }
}