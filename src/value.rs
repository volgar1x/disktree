use std::ops::Range;

pub(crate) const VALUE_LENGTH: usize = 12;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Value {
    pub offset: u32,
    pub length: u32,
    pub decompressed: u32,
}

impl Value {
    pub fn from_bytes(bytes: &[u8; VALUE_LENGTH]) -> Self {
        let halves: &[[u8; 4]; 3] = unsafe { &*bytes.as_ptr().cast() };
        let [offset, length, decompressed] = halves;
        Self {
            offset: u32::from_be_bytes(*offset),
            length: u32::from_be_bytes(*length),
            decompressed: u32::from_be_bytes(*decompressed),
        }
    }

    pub fn into_bytes(self) -> [u8; VALUE_LENGTH] {
        let bytes: [[u8; 4]; 3] = [
            self.offset.to_be_bytes(),
            self.length.to_be_bytes(),
            self.decompressed.to_be_bytes(),
        ];
        unsafe { std::mem::transmute(bytes) }
    }

    pub fn into_range32(self) -> Range<u32> {
        self.offset..(self.offset + self.length)
    }

    pub fn into_range(self) -> Range<usize> {
        let start: usize = self.offset.try_into().unwrap();
        let length: usize = self.length.try_into().unwrap();
        start..(start + length)
    }

    pub fn with_offset(self, offset: u32) -> Self {
        Self { offset, ..self }
    }
}
