use widestring::U16CStr;
use windows::{
    Devices::{
        Bluetooth::{
            BluetoothLEDevice,
            GenericAttributeProfile::{GattDeviceService, GattServiceUuids},
        },
        Enumeration::{DeviceClass, DeviceInformation},
    },
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::{
                CM_LOCATE_DEVNODE_FLAGS, CM_LOCATE_DEVNODE_NORMAL, CM_Locate_DevNodeW, CR_SUCCESS,
                DIGCF_ALLCLASSES, DIGCF_PRESENT, SP_DEVINFO_DATA, SetupDiDestroyDeviceInfoList,
                SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiGetDeviceInstanceIdW,
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

fn is_airpods(device: &IMMDevice) -> Result<bool> {
    unsafe {
        // Open the property store for the device
        let store = device.OpenPropertyStore(STGM_READ)?;

        // Get the friendly name
        let prop = store.GetValue(&PKEY_Device_MatchingDeviceId)?;

        // prop contains a multi-string (vector of strings)
        let vt = prop.Anonymous.Anonymous.vt.0;
        println!("vt={vt}");

        if vt != 0 {
            println!(
                "{:?}",
                prop.Anonymous.Anonymous.Anonymous.pwszVal.to_string()
            );
        }

        // if (vt & VT_VECTOR.0) != 0 && (vt & VT_LPWSTR.0) != 0 {
        //     let calpwstr = prop.Anonymous.Anonymous.Anonymous.calpwstr;
        //     let count = calpwstr.cElems as usize;
        //     let ptrs = calpwstr.pElems;

        //     for i in 0..count {
        //         let wide_ptr = *ptrs.add(i);
        //         if !wide_ptr.is_null() {
        //             let s = U16CStr::from_ptr_str(wide_ptr.0); // extract raw pointer
        //             println!("{}", s.to_string_lossy());
        //         }
        //     }
        // } else {
        // }
        Ok(false)
    }
}

fn find_airpods() -> windows::core::Result<()> {
    unsafe {
        let hdev = SetupDiGetClassDevsW(None, None, None, DIGCF_ALLCLASSES | DIGCF_PRESENT)?;
        let mut index = 0;
        loop {
            let mut info = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            if !SetupDiEnumDeviceInfo(hdev, index, &mut info).is_ok() {
                break;
            }

            let mut buffer = [0u16; 512];
            let mut required_size = 0u32;
            if SetupDiGetDeviceInstanceIdW(
                hdev,
                &mut info,
                Some(&mut buffer),
                Some(&mut required_size),
            )
            .is_ok()
            {
                let s = String::from_utf16_lossy(&buffer[..required_size as usize - 1]);
                if s.to_lowercase()
                    .contains("74ec2172-0bad-4d01-8f77-997b2be0722a")
                {
                    println!("Found AirPods: {}", s);
                }
            }

            index += 1;
        }

        let _ = SetupDiDestroyDeviceInfoList(hdev);
    }
    Ok(())
}

fn get_parent_device_guid(device_instance: &str) -> Option<String> {
    unsafe {
        let hdev = SetupDiGetClassDevsW(None, None, None, DIGCF_ALLCLASSES | DIGCF_PRESENT).ok()?;
        let mut index = 0;

        loop {
            let mut info = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            if !SetupDiEnumDeviceInfo(hdev, index, &mut info).is_ok() {
                break;
            }

            let mut buffer = [0u16; 512];
            let mut needed = 0;
            if SetupDiGetDeviceInstanceIdW(hdev, &mut info, Some(&mut buffer), Some(&mut needed))
                .is_ok()
            {
                let s = String::from_utf16_lossy(&buffer[..needed as usize - 1]);
                if s.to_lowercase()
                    .contains("74ec2172-0bad-4d01-8f77-997b2be0722a")
                {
                    return Some(s);
                }
            }
            index += 1;
        }

        let _ = SetupDiDestroyDeviceInfoList(hdev);
    }
    None
}

fn find_airpods_devinst() -> Option<u32> {
    unsafe {
        let hdev = SetupDiGetClassDevsW(None, None, None, DIGCF_ALLCLASSES | DIGCF_PRESENT).ok()?;
        let mut index = 0;

        loop {
            let mut info = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            if !SetupDiEnumDeviceInfo(hdev, index, &mut info).is_ok() {
                break;
            }

            let mut buffer = [0u16; 512];
            let mut needed = 0;
            if SetupDiGetDeviceInstanceIdW(hdev, &mut info, Some(&mut buffer), Some(&mut needed))
                .is_ok()
            {
                let s = String::from_utf16_lossy(&buffer[..needed as usize - 1]);
                if s.to_lowercase()
                    .contains("74ec2172-0bad-4d01-8f77-997b2be0722a")
                {
                    let _ = SetupDiDestroyDeviceInfoList(hdev);
                    return Some(info.DevInst);
                }
            }
            index += 1;
        }
        let _ = SetupDiDestroyDeviceInfoList(hdev);
    }
    None
}

fn device_id_to_devinst(endpoint_id: PWSTR) -> Option<u32> {
    unsafe {
        let mut devinst: u32 = 0;
        // Convert endpoint ID to device instance path if necessary
        // CM_Locate_DevNode expects the device instance ID
        if CM_Locate_DevNodeW(
            &mut devinst,
            PWSTR(endpoint_id.as_ptr() as *mut _),
            CM_LOCATE_DEVNODE_NORMAL,
        ) == CR_SUCCESS
        {
            Some(devinst)
        } else {
            None
        }
    }
}

use windows::Win32::Devices::DeviceAndDriverInstallation::*;
use windows::Win32::Foundation::*;
use windows::Win32::Media::Audio::*;

fn endpoint_belongs_to_airpods(device: &IMMDevice, airpods_devinst: u32) -> bool {
    unsafe {
        // Get the endpoint ID
        let id = device.GetId().ok();
        if id.is_none() {
            return false;
        }
        let id = id.unwrap();
        let s = U16CStr::from_ptr_str(id.0);
        let endpoint_id = s.to_string_lossy();

        // Enumerate all PnP devices and find devnode matching endpoint_id
        let hdev = SetupDiGetClassDevsW(None, None, None, DIGCF_ALLCLASSES | DIGCF_PRESENT).ok();
        if hdev.is_none() {
            return false;
        }
        let hdev = hdev.unwrap();

        let mut index = 0;
        loop {
            let mut info = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            if !SetupDiEnumDeviceInfo(hdev, index, &mut info).is_ok() {
                break;
            }

            let mut buffer = [0u16; 512];
            let mut needed = 0;
            if SetupDiGetDeviceInstanceIdW(hdev, &mut info, Some(&mut buffer), Some(&mut needed))
                .is_ok()
            {
                let dev_inst_id = String::from_utf16_lossy(&buffer[..needed as usize - 1]);

                // Check if this device is a child of the AirPods DEVINST
                let mut parent = 0;
                if CM_Get_Parent(&mut parent, info.DevInst, 0) == CR_SUCCESS {
                    if parent == airpods_devinst {
                        let _ = SetupDiDestroyDeviceInfoList(hdev);
                        println!("dev_inst_id: {:?}", dev_inst_id);
                        return true;
                    }
                }
            }

            index += 1;
        }

        let _ = SetupDiDestroyDeviceInfoList(hdev);
        false
    }
}

#[implement(IMMNotificationClient)]
struct AudioNotif;

impl IMMNotificationClient_Impl for AudioNotif_Impl {
    fn OnDeviceStateChanged(&self, device_id: &PCWSTR, new_state: DEVICE_STATE) -> Result<()> {
        println!(
            "Device state changed: 0x{:X}, device_id={:?}",
            new_state.0, device_id
        );
        // let enumerator: IMMDeviceEnumerator =
        //     unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        // let device = unsafe { enumerator.GetDevice(*device_id)? };

        // let wide_ptr = unsafe { device.GetId() }?;
        // let s = unsafe { U16CStr::from_ptr_str(wide_ptr.0) };
        // let endpoint_id = s.to_string_lossy();

        // let res = find_airpods_devinst();

        // if let Some(res) = res {
        //     if endpoint_belongs_to_airpods(&device, res) {
        //         print!("SAME");
        //     } else {
        //         println!("NOT");
        //     }
        // }

        // println!("{:?}", endpoint_id);

        // if is_airpods(&device)? {
        //     println!("AirPods connected: {:?}", device_id);
        // }
        Ok(())
    }

    fn OnDeviceAdded(&self, device_id: &PCWSTR) -> Result<()> {
        println!("Device added: {:?}", device_id);
        // let enumerator: IMMDeviceEnumerator =
        //     unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        // let device = unsafe { enumerator.GetDevice(*device_id)? };

        // let wide_ptr = unsafe { device.GetId() }?;
        // let s = unsafe { U16CStr::from_ptr_str(wide_ptr.0) }; // extract raw pointer
        // println!("{}", s.to_string_lossy());
        // if is_airpods(&device)? {
        //     println!("AirPods connected: {:?}", device_id);
        // }
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
        println!(
            "Property value changed: key={:?}, device_id={:?}",
            *key, device_id
        );
        Ok(())
    }
}

fn main() -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let callback = AudioNotif;
        let client: IMMNotificationClient = callback.into();

        enumerator.RegisterEndpointNotificationCallback(&client)?;

        println!("Listening for audio endpoint changes.");

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

// #[tokio::main]
// async fn main() -> windows::core::Result<()> {
//     // Filter for all classic Bluetooth devices (BTHENUM)
//     let selector: HSTRING = "System.Devices.DeviceInstanceId:~=\"BTHENUM\"".into();

//     let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.await?;

//     for dev in devices {
//         println!("Name: {:?}", dev.Name()?);
//         println!("ID:   {:?}", dev.Id()?);
//     }

//     Ok(())
// }
