use crate::airpod::{
    Model,
    packet::{Battery, Color, Flags, Lid, PacketType},
};

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

pub const VENDOR_ID: u16 = 76;

pub fn as_airpods(data: &[u8]) -> Option<AirPods> {
    if !AirPods::is_valid(data) || data.len() < std::mem::size_of::<AirPods>() {
        return None;
    }
    let mut ap = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const AirPods) };
    ap.color = Color::from(ap.color as u8);
    ap.packet_type = PacketType::from(ap.packet_type as u8);
    Some(ap)
}

impl AirPods {
    pub fn is_valid(data: &[u8]) -> bool {
        if data.len() != std::mem::size_of::<AirPods>() {
            return false;
        }
        data[0] == PacketType::ProximityPairing as u8 && data[1] == (27 - 2) // remaining length
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
