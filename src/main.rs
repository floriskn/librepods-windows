#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use windows::Storage::Streams::{DataReader, IBuffer};
use windows::{
    Devices::Bluetooth::Advertisement::{
        BluetoothLEAdvertisementReceivedEventArgs, BluetoothLEAdvertisementWatcher,
    },
    Foundation::TypedEventHandler,
    core::Result,
};

use crate::airpod::{AirPods, VENDOR_ID, as_airpods};

mod airpod;

/// Checks if the advertisement contains AirPods manufacturer data
pub fn is_desired_adv(manufacturer_data_map: &HashMap<u16, Vec<u8>>) -> bool {
    match manufacturer_data_map.get(&VENDOR_ID) {
        Some(data) => AirPods::is_valid(data),
        None => false,
    }
}

/// Converts a Windows IBuffer to a Vec<u8>
fn buffer_to_vec(buffer: &IBuffer) -> Result<Vec<u8>> {
    let reader = DataReader::FromBuffer(buffer)?;
    let mut bytes = vec![0u8; reader.UnconsumedBufferLength()? as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(bytes)
}

fn main() -> Result<()> {
    // Shared map for storing manufacturer data
    let manufacturer_map: Arc<Mutex<HashMap<u16, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
    let map_clone = manufacturer_map.clone();

    let watcher = BluetoothLEAdvertisementWatcher::new()?;

    // Event handler for received BLE advertisements
    let _token = watcher.Received(&TypedEventHandler::new(
        move |_: windows::core::Ref<BluetoothLEAdvertisementWatcher>,
              args: windows::core::Ref<BluetoothLEAdvertisementReceivedEventArgs>| {
            if let Some(args) = &*args {
                let manufacturer_data_array = args.Advertisement()?.ManufacturerData()?;
                let size = manufacturer_data_array.Size()?;

                for i in 0..size {
                    let manufacturer_data = manufacturer_data_array.GetAt(i)?;
                    let company_id = manufacturer_data.CompanyId()?;
                    let data_vec = buffer_to_vec(&manufacturer_data.Data()?)?;

                    map_clone.lock().unwrap().insert(company_id, data_vec);
                }

                // Try parsing AirPods data
                if let Some(data) = map_clone.lock().unwrap().get(&VENDOR_ID) {
                    if let Some(airpod) = as_airpods(data) {
                        let info = airpod.debug_info();
                        print!("\r{}", info);
                        io::stdout().flush().unwrap();
                    }
                }
            }
            Ok(())
        },
    ))?;

    watcher.Start()?;
    println!("Watching for BLE advertisements...");
    thread::sleep(Duration::from_secs(30)); // run for 30 seconds
    watcher.Stop()?;

    Ok(())
}
