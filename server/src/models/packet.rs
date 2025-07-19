pub struct Packet<'a> {
    message: &'a str,
}

impl<'a> Packet<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }

    pub fn to_buff(self) -> [u8; 1024] {
        let mut buff_to_fill: [u8; 1024] = [0; 1024];
        buff_to_fill[..self.message.len()].copy_from_slice(&self.message.as_bytes());

        buff_to_fill
    }
}
