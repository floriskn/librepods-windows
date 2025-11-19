#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PacketType {
    Unknown = 0xFF,
    AirPrint = 0x3,
    AirDrop = 0x5,
    HomeKit = 0x6,
    ProximityPairing = 0x7,
    HeySiri = 0x8,
    AirPlay = 0x9,
    MagicSwitch = 0xB,
    Handoff = 0xC,
    InstantHotspotTetheringTargetPresence = 0xD,
    InstantHotspotTetheringSourcePresence = 0xE,
    NearbyAction = 0xF,
    NearbyInfo = 0x10,
}

impl From<u8> for PacketType {
    fn from(val: u8) -> Self {
        match val {
            0x3 => PacketType::AirPrint,
            0x5 => PacketType::AirDrop,
            0x6 => PacketType::HomeKit,
            0x7 => PacketType::ProximityPairing,
            0x8 => PacketType::HeySiri,
            0x9 => PacketType::AirPlay,
            0xB => PacketType::MagicSwitch,
            0xC => PacketType::Handoff,
            0xD => PacketType::InstantHotspotTetheringTargetPresence,
            0xE => PacketType::InstantHotspotTetheringSourcePresence,
            0xF => PacketType::NearbyAction,
            0x10 => PacketType::NearbyInfo,
            _ => PacketType::Unknown,
        }
    }
}
