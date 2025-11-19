use crate::airpod::packet::PacketType;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub packet_type: PacketType,
    pub remaining_length: u8,
}
