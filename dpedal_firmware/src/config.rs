use dpedal_config::{ArchivedConfig, CONFIG_OFFSET, CONFIG_SIZE, Config, RP2040_FLASH_OFFSET};
use rkyv::{rancor::Failure, util::Align};

// TODO: store in heap instead, apparently only 2kb of stack o.0

pub fn load() -> Result<Config, Failure> {
    let bytes = load_config_bytes_from_flash();
    let size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let archive = rkyv::api::low::access::<ArchivedConfig, Failure>(&(&*bytes)[4..4 + size])?;
    rkyv::api::low::deserialize::<_, Failure>(archive)
}

pub fn load_config_bytes_from_flash() -> Align<[u8; CONFIG_SIZE]> {
    let mut data = Align([0; CONFIG_SIZE]);
    // Safety: This byte range is known to be valid flash memory on this device
    unsafe {
        for i in 0..CONFIG_SIZE {
            let address = (RP2040_FLASH_OFFSET + CONFIG_OFFSET + i) as *mut u8;
            data[i] = core::ptr::read_volatile(address);
        }
    }
    data
}
