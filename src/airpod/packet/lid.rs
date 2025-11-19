#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct Lid {
    pub bits: u8,
}

impl Lid {
    pub fn switch_count(&self) -> u8 {
        self.bits & 0b0000_0111
    }
    pub fn closed(&self) -> bool {
        self.bits & 0b0000_1000 != 0
    }
}
