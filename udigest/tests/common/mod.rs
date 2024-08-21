/// A buffer based on `Vec<u8>`. Writing to the buffer
/// appends data to the vector
pub struct VecBuf(pub Vec<u8>);

impl udigest::encoding::Buffer for VecBuf {
    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes)
    }
}

/// Encodes digestable data into bytes
pub fn encode_to_vec(x: &impl udigest::Digestable) -> Vec<u8> {
    let mut buffer = VecBuf(vec![]);
    x.unambiguously_encode(udigest::encoding::EncodeValue::new(&mut buffer));
    buffer.0
}
