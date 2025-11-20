use std::sync::{Arc, Mutex};

use widestring::U16CStr;
use windows::{
    Devices::{
        Bluetooth::{
            BluetoothLEDevice,
            GenericAttributeProfile::{GattDeviceService, GattServiceUuids, GattWriteOption},
        },
        Enumeration::{DeviceClass, DeviceInformation},
    },
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::{
                CM_Get_Parent, CM_LOCATE_DEVNODE_FLAGS, CM_LOCATE_DEVNODE_NORMAL,
                CM_Locate_DevNodeW, CR_SUCCESS, DIGCF_ALLCLASSES, DIGCF_PRESENT, SP_DEVINFO_DATA,
                SPDRP_FRIENDLYNAME, SPDRP_HARDWAREID, SPDRP_LOCATION_INFORMATION,
                SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
                SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceRegistryPropertyW,
            },
            FunctionDiscovery::*,
        },
        Foundation::*,
        Media::Audio::*,
        System::{
            Com::{StructuredStorage::PROPVARIANT, *},
            Variant::{VT_LPWSTR, VT_VECTOR},
        },
    },
};
use windows_core::*;

struct AirPodsState {
    devinst: Option<u32>,
    ble_address: Option<u64>,
}

#[implement(IMMNotificationClient)]
struct AudioNotif {
    state: Arc<Mutex<AirPodsState>>,
}

impl IMMNotificationClient_Impl for AudioNotif_Impl {
    fn OnDeviceStateChanged(&self, device_id: &PCWSTR, new_state: DEVICE_STATE) -> Result<()> {
        println!(
            "Device state changed: 0x{:X}, device_id={:?}",
            new_state.0, device_id
        );

        let s = unsafe { device_id.to_string() }?;

        if let Some(devinst) = find_airpods_devinst_from_endpoint(&s) {
            println!("Mapped to DEVINST: {}", devinst);

            if let Some(s) = devinst_to_ble_address(devinst) {
                println!("BLE {}", s);
            }
        } else {
            println!("Could not map device_id to DEVINST");
        }

        Ok(())
    }

    fn OnDeviceAdded(&self, device_id: &PCWSTR) -> Result<()> {
        println!("Device added: {:?}", device_id);

        let s = unsafe { device_id.to_string() }?;

        if let Some(devinst) = find_airpods_devinst_from_endpoint(&s) {
            println!("Mapped to DEVINST: {}", devinst);
        }
        Ok(())
    }

    fn OnDeviceRemoved(&self, device_id: &PCWSTR) -> Result<()> {
        println!("Device removed: {:?}", device_id);
        Ok(())
    }

    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        device_id: &PCWSTR,
    ) -> Result<()> {
        println!(
            "Default device changed: flow={:?}, role={:?}, device_id={:?}",
            flow, role, device_id
        );

        Ok(())
    }

    fn OnPropertyValueChanged(&self, device_id: &PCWSTR, key: &PROPERTYKEY) -> Result<()> {
        // println!(
        //     "Device state changed: 0x{:X}, device_id={:?}",
        //     new_state.0, device_id
        // );
        Ok(())
    }
}

/// Given an IMMDevice ID string, find the DEVINST of the corresponding AirPods/Bluetooth device
fn find_airpods_devinst_from_endpoint(device_id: &str) -> Option<u32> {
    // Extract the GUID part from the IMMDevice ID
    // Example: "{0.0.0.00000000}.{74ec2172-0bad-4d01-8f77-997b2be0722a}"
    let guid_part = device_id
        .split('.')
        .last()?
        .trim_matches(|c| c == '{' || c == '}');

    let mut devinst: u32 = 0;
    let hdev =
        unsafe { SetupDiGetClassDevsW(None, None, None, DIGCF_PRESENT | DIGCF_ALLCLASSES) }.ok()?;

    let mut index = 0;
    loop {
        let mut info = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };

        let success = unsafe { SetupDiEnumDeviceInfo(hdev, index, &mut info).is_ok() };
        if !success {
            break;
        }

        // Get the device instance ID
        let mut buffer = [0u16; 512];
        let mut required_size = 0;
        let res =
            unsafe { SetupDiGetDeviceInstanceIdW(hdev, &mut info, None, Some(&mut required_size)) };

        if required_size == 0 {
            index += 1;
            continue;
        }
        let mut buffer: Vec<u16> = vec![0; required_size as usize];
        let success2 = unsafe {
            SetupDiGetDeviceInstanceIdW(
                hdev,
                &mut info,
                Some(&mut buffer),
                Some(&mut required_size),
            )
        };
        if success2.is_ok() {
            let devinst_id = unsafe { U16CStr::from_ptr_str(buffer.as_ptr()) };
            let devinst_id_str = devinst_id.to_string_lossy().to_lowercase();

            if devinst_id_str.contains(&guid_part.to_lowercase()) {
                // Found matching DEVINST
                devinst = info.DevInst;
                break;
            }
        }

        index += 1;
    }

    unsafe { SetupDiDestroyDeviceInfoList(hdev) };
    if devinst != 0 { Some(devinst) } else { None }
}

fn devinst_to_ble_address(devinst: u32) -> Option<u64> {
    let mut buffer = [0u8; 8];
    let mut needed: u32 = 0;

    unsafe {
        let hdev = SetupDiGetClassDevsW(None, None, None, DIGCF_PRESENT | DIGCF_ALLCLASSES).ok()?;

        let mut info = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        info.DevInst = devinst;

        // SPDRP_BLUETOOTH_ADDRESS is not a constant in windows crate; you may need its property ID manually
        if SetupDiGetDeviceRegistryPropertyW(
            hdev,
            &mut info,
            SPDRP_FRIENDLYNAME, // example: replace with actual Bluetooth address property ID
            None,
            None,
            Some(&mut needed),
        )
        .is_ok()
        {
            Some(u64::from_le_bytes(buffer))
        } else {
            None
        }
    }
}

// fn main() -> Result<()> {
//     unsafe {
//         let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

//         let state = Arc::new(Mutex::new(AirPodsState {
//             devinst: None,
//             ble_address: None,
//         }));

//         let enumerator: IMMDeviceEnumerator =
//             CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

//         let callback = AudioNotif {
//             state: state.clone(),
//         };
//         let client: IMMNotificationClient = callback.into();

//         enumerator.RegisterEndpointNotificationCallback(&client)?;

//         println!("Listening for audio endpoint changes.");

//         loop {
//             std::thread::sleep(std::time::Duration::from_secs(1));
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    let mac_str = "2C7600C10DA6"; // from registry
    let bytes: Vec<u8> = (0..mac_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&mac_str[i..i + 2], 16).unwrap())
        .collect();
    let ble_address = u64::from_le_bytes([
        bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0], 0, 0,
    ]);

    println!("Connecting to device {:X}...", ble_address);

    let selector = BluetoothLEDevice::GetDeviceSelector()?;
    let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.await?;

    println!("{}", devices.Size()?);

    for info in &devices {
        let id = info.Id()?;
        let dev = BluetoothLEDevice::FromIdAsync(&id)?.await?;
        let name = match dev.Name() {
            Ok(n) => {
                println!("Device name: {:?}", n);
                n
            }
            Err(e) => {
                println!("Failed to get device name: {:?}", e);
                HSTRING::from("<unknown>")
            }
        };
    }

    let device_res = BluetoothLEDevice::FromBluetoothAddressAsync(ble_address);
    println!("Called FromBluetoothAddressAsync");

    let device = match device_res {
        Ok(future) => {
            println!("Awaiting device future...");
            match future.await {
                Ok(d) => {
                    println!("Device connected");
                    d
                }
                Err(e) => {
                    println!("Failed to await device: {:?}", e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!("Failed to call FromBluetoothAddressAsync: {:?}", e);
            return Err(e);
        }
    };

    let properties = device.DeviceInformation()?.Properties()?;

    let it = properties.into_iter();
    for kv in it {
        println!("{} ", kv.Key()?);
    }

    let name = match device.Name() {
        Ok(n) => {
            println!("Device name: {:?}", n);
            n
        }
        Err(e) => {
            println!("Failed to get device name: {:?}", e);
            HSTRING::from("<unknown>")
        }
    };

    println!("Connected to device: {:?}", name);

    let services_future = device.GetGattServicesAsync();
    println!("Called GetGattServicesAsync");

    let services_result = match services_future {
        Ok(fut) => {
            println!("Awaiting services...");
            match fut.await {
                Ok(sr) => {
                    println!("Got services result");
                    sr
                }
                Err(e) => {
                    println!("Failed to await services: {:?}", e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!("Failed to call GetGattServicesAsync: {:?}", e);
            return Err(e);
        }
    };

    let services = match services_result.Services() {
        Ok(s) => {
            println!("Retrieved services collection: {} services", s.Size()?);
            s
        }
        Err(e) => {
            println!("Failed to get services collection: {:?}", e);
            return Err(e);
        }
    };

    for service in services {
        match service.Uuid() {
            Ok(uuid) => println!("Service UUID: {:?}", uuid),
            Err(e) => println!("Failed to get service UUID: {:?}", e),
        }

        let chars_future = service.GetCharacteristicsAsync();
        println!("Called GetCharacteristicsAsync for service");

        let characteristics_result = match chars_future {
            Ok(fut) => match fut.await {
                Ok(cr) => cr,
                Err(e) => {
                    println!("Failed to await characteristics: {:?}", e);
                    continue;
                }
            },
            Err(e) => {
                println!("Failed to call GetCharacteristicsAsync: {:?}", e);
                continue;
            }
        };

        let characteristics = match characteristics_result.Characteristics() {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to get characteristics collection: {:?}", e);
                continue;
            }
        };

        for ch in characteristics {
            match ch.Uuid() {
                Ok(ch_uuid) => println!("  Characteristic UUID: {:?}", ch_uuid),
                Err(e) => println!("  Failed to get characteristic UUID: {:?}", e),
            }
        }
    }

    println!("Finished enumerating characteristics");
    Ok(())
}
