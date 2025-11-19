#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct Battery {
    pub bits: u8,
    pub extra: u8,
}

impl Battery {
    pub fn curr(&self) -> u8 {
        self.bits & 0x0F
    }
    pub fn anot(&self) -> u8 {
        (self.bits >> 4) & 0x0F
    }
    pub fn case_box(&self) -> u8 {
        self.extra & 0x0F
    }
    pub fn curr_charging(&self) -> bool {
        (self.extra & 0b0001_0000) != 0
    }
    pub fn anot_charging(&self) -> bool {
        (self.extra & 0b0010_0000) != 0
    }
    pub fn case_charging(&self) -> bool {
        (self.extra & 0b0100_0000) != 0
    }
}
