#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct Flags {
    pub bits: u8,
}

impl Flags {
    pub fn curr_in_ear(&self) -> bool {
        self.bits & 0b0000_0010 != 0
    }
    pub fn both_in_case(&self) -> bool {
        self.bits & 0b0000_0100 != 0
    }
    pub fn anot_in_ear(&self) -> bool {
        self.bits & 0b0000_1000 != 0
    }
    pub fn broadcast_from(&self) -> bool {
        self.bits & 0b0010_0000 != 0
    }
}
