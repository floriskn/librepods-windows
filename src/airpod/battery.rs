#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery(Option<u32>);

impl Battery {
    pub fn new() -> Self {
        Battery(None)
    }
    pub fn from_value(value: u32) -> Self {
        Battery(Some(value))
    }

    pub fn available(&self) -> bool {
        self.0.is_some()
    }

    pub fn value(&self) -> u32 {
        self.0.unwrap_or_else(|| {
            // log warning in Rust style if needed
            eprintln!("Trying to get the battery value but unavailable.");
            0
        })
    }

    pub fn is_low_battery(&self) -> bool {
        self.0.map(|v| v <= 20).unwrap_or_else(|| {
            eprintln!("Trying to determine that the battery is low but unavailable.");
            false
        })
    }
}
