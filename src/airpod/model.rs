#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    Unknown = 0,
    AirPods1,
    AirPods2,
    AirPods3,
    AirPodsPro,
    AirPodsPro2,
    AirPodsPro2UsbC,
    AirPodsMax,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Model::AirPods1 => "AirPods 1",
            Model::AirPods2 => "AirPods 2",
            Model::AirPods3 => "AirPods 3",
            Model::AirPodsPro => "AirPods Pro",
            Model::AirPodsPro2 => "AirPods Pro 2",
            Model::AirPodsPro2UsbC => "AirPods Pro 2 (USB-C)",
            Model::AirPodsMax => "AirPods Max",
            Model::Unknown => "Unknown",
        }
    }
}
