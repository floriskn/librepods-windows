use crate::airpod::Model;

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
            _ => PacketType::Unknown, // fallback default
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Unknown = 0xFF, // fallback
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub packet_type: PacketType,
    pub remaining_length: u8,
}
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct Flags {
    pub bits: u8, // entire byte stores all 8 bitfields
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
    // add other getters as needed
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct Battery {
    pub bits: u8,  // first 8 bits
    pub extra: u8, // next 8 bits, total 16 bits for all battery info
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

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct Lid {
    pub bits: u8, // single byte storing switchCount, closed, unk10
}

impl Lid {
    pub fn switch_count(&self) -> u8 {
        self.bits & 0b0000_0111
    }
    pub fn closed(&self) -> bool {
        self.bits & 0b0000_1000 != 0
    }
}

pub const VENDOR_ID: u16 = 76;

// AirPods struct as close as possible to C++ layout
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct AirPods {
    pub packet_type: PacketType,
    pub remaining_length: u8,
    pub unk1: [u8; 1],
    pub model_id: u16,
    pub flags: Flags,
    pub battery: Battery,
    pub lid: Lid,
    pub color: Color,
    pub unk11: u8,
    pub unk12: [u8; 16],
}

// Implement size check like static_assert
const _: () = assert!(std::mem::size_of::<AirPods>() == 27);

// Standalone function like template As<T>
pub fn as_airpods(data: &[u8]) -> Option<AirPods> {
    if !AirPods::is_valid(data) || data.len() < std::mem::size_of::<AirPods>() {
        return None;
    }

    // Unsafe read of bytes into struct
    let mut ap = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const AirPods) };

    // Normalize enum fields
    ap.color = Color::from(ap.color as u8);
    ap.packet_type = PacketType::from(ap.packet_type as u8);

    Some(ap)
}

impl AirPods {
    pub fn is_valid(data: &[u8]) -> bool {
        if data.len() != std::mem::size_of::<AirPods>() {
            return false;
        }
        let should_remaining_length = 27 - 2; // sizeof(AirPods) - offset_of(remaining_length) - 1
        data[0] == PacketType::ProximityPairing as u8 && data[1] == should_remaining_length as u8
    }

    pub fn get_model(model_id: u16) -> Model {
        match model_id {
            0x2002 => Model::AirPods1,
            0x200F => Model::AirPods2,
            0x2013 => Model::AirPods3,
            0x200E => Model::AirPodsPro,
            0x2014 => Model::AirPodsPro2,
            0x2024 => Model::AirPodsPro2UsbC,
            0x200A => Model::AirPodsMax,
            _ => Model::Unknown,
        }
    }

    fn broadcast_side(&self) -> bool {
        self.flags.broadcast_from()
    }
    pub fn is_left_broadcasted(&self) -> bool {
        self.broadcast_side()
    }
    pub fn is_right_broadcasted(&self) -> bool {
        !self.broadcast_side()
    }

    pub fn get_model_instance(&self) -> Model {
        Self::get_model(self.model_id)
    }

    pub fn left_battery(&self) -> u8 {
        let val = if self.is_left_broadcasted() {
            self.battery.curr()
        } else {
            self.battery.anot()
        };
        // TODO: make option
        if val <= 10 { val } else { 0 }
    }

    pub fn right_battery(&self) -> u8 {
        let val = if self.is_right_broadcasted() {
            self.battery.curr()
        } else {
            self.battery.anot()
        };
        // TODO: make option
        if val <= 10 { val } else { 0 }
    }

    pub fn case_battery(&self) -> u8 {
        if self.battery.case_box() <= 10 {
            self.battery.case_box()
        } else {
            0
        }
    }

    pub fn is_left_charging(&self) -> bool {
        if self.is_left_broadcasted() {
            self.battery.curr_charging()
        } else {
            self.battery.anot_charging()
        }
    }

    pub fn is_right_charging(&self) -> bool {
        if self.is_right_broadcasted() {
            self.battery.curr_charging()
        } else {
            self.battery.anot_charging()
        }
    }

    pub fn is_both_in_case(&self) -> bool {
        self.flags.both_in_case()
    }
    pub fn is_lid_opened(&self) -> bool {
        !self.lid.closed()
    }
    pub fn is_case_charging(&self) -> bool {
        self.battery.case_charging()
    }

    pub fn is_left_in_ear(&self) -> bool {
        !self.is_left_charging()
            && if self.is_left_broadcasted() {
                self.flags.curr_in_ear()
            } else {
                self.flags.anot_in_ear()
            }
    }
    pub fn is_right_in_ear(&self) -> bool {
        !self.is_right_charging()
            && if self.is_right_broadcasted() {
                self.flags.curr_in_ear()
            } else {
                self.flags.anot_in_ear()
            }
    }

    pub fn desensitize(&self) -> Self {
        let mut result = *self;
        result.unk12 = [0u8; 16];
        result
    }

    pub fn debug_info(&self) -> String {
        let model = self.get_model_instance().as_str();
        // Battery x10
        let left_batt = self.left_battery() * 10;
        let right_batt = self.right_battery() * 10;
        let case_batt = self.case_battery() * 10;
        let left_charging = self.is_left_charging();
        let right_charging = self.is_right_charging();
        let case_charging = self.is_case_charging();
        let both_in_case = self.is_both_in_case();
        let lid_opened = self.is_lid_opened();
        let left_in_ear = self.is_left_in_ear();
        let right_in_ear = self.is_right_in_ear();
        let color = format!("{:?}", self.color);
        let packet_type = format!("{:?}", self.packet_type);
        let remaining_length = self.remaining_length;

        format!(
            "AirPods Debug Info:\n\
            Model: {}\n\
            Packet Type: {}\n\
            Remaining Length: {}\n\
            Color: {}\n\
            Left Battery: {}{}\n\
            Right Battery: {}{}\n\
            Case Battery: {}{}\n\
            Both in Case: {}\n\
            Lid Opened: {}\n\
            Left In Ear: {}\n\
            Right In Ear: {}\n\
            Desensitized Payload: {:02X?}",
            model,
            packet_type,
            remaining_length,
            color,
            left_batt,
            if left_charging { " (charging)" } else { "" },
            right_batt,
            if right_charging { " (charging)" } else { "" },
            case_batt,
            if case_charging { " (charging)" } else { "" },
            both_in_case,
            lid_opened,
            left_in_ear,
            right_in_ear,
            self.unk12
        )
    }
}
