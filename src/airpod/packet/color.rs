#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Unknown = 0xFF,
    White = 0x0,
    Black = 0x1,
    Red = 0x2,
    Blue = 0x3,
    Pink = 0x4,
    Gray = 0x5,
    Silver = 0x6,
    Gold = 0x7,
    RoseGold = 0x8,
    SpaceGray = 0x9,
    DarkBlue = 0xA,
    LightBlue = 0xB,
    Yellow = 0xC,
}

impl From<u8> for Color {
    fn from(val: u8) -> Self {
        match val {
            0x0 => Color::White,
            0x1 => Color::Black,
            0x2 => Color::Red,
            0x3 => Color::Blue,
            0x4 => Color::Pink,
            0x5 => Color::Gray,
            0x6 => Color::Silver,
            0x7 => Color::Gold,
            0x8 => Color::RoseGold,
            0x9 => Color::SpaceGray,
            0xA => Color::DarkBlue,
            0xB => Color::LightBlue,
            0xC => Color::Yellow,
            _ => Color::Unknown,
        }
    }
}
