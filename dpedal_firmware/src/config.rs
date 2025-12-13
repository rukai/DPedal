use arrayvec::ArrayVec;
use defmt::error;
use dpedal_config::{
    ArchivedConfig, CONFIG_OFFSET, CONFIG_SIZE, Config, RP2040_FLASH_OFFSET, RP2040_FLASH_SIZE,
};
use embassy_rp::{
    Peri,
    flash::{Blocking, Flash},
    peripherals::FLASH,
};
use rkyv::{rancor::Failure, util::Align};

pub struct ConfigFlash {
    flash: Flash<'static, FLASH, Blocking, RP2040_FLASH_SIZE>,
}

impl ConfigFlash {
    pub fn new(p_flash: Peri<'static, FLASH>) -> Self {
        ConfigFlash {
            flash: Flash::new_blocking(p_flash),
        }
    }

    pub fn load(&self) -> Result<Config, ()> {
        let bytes = self.load_config_bytes_from_flash()?;
        let archive = rkyv::api::low::access::<ArchivedConfig, Failure>(&bytes).map_err(|_| ())?;
        rkyv::api::low::deserialize::<_, Failure>(archive).map_err(|_| ())
    }

    pub fn load_config_bytes_from_flash(&self) -> Result<Align<ArrayVec<u8, CONFIG_SIZE>>, ()> {
        let mut data = Align(ArrayVec::new());
        // Safety: This byte range is known to be valid flash memory on this device
        unsafe {
            let mut size = 0u32;
            for i in 0..4 {
                let address = (RP2040_FLASH_OFFSET + CONFIG_OFFSET + i) as *mut u8;
                size |= (core::ptr::read_volatile::<u8>(address) as u32) << ((3 - i) * 8);
            }

            let size = size as usize;
            if size > CONFIG_SIZE {
                error!("config bytes length prefix too long {}", size);
                return Err(());
            }

            for i in 0..size {
                let address = (RP2040_FLASH_OFFSET + CONFIG_OFFSET + 4 + i) as *mut u8;
                data.push(core::ptr::read_volatile::<u8>(address));
            }
        }
        Ok(data)
    }

    // TODO: store in heap instead, apparently only 2kb of stack o.0

    pub fn check_valid_config(&self, bytes: &[u8]) -> Result<(), ()> {
        let archive = rkyv::api::low::access::<ArchivedConfig, Failure>(bytes).map_err(|_| ())?;
        rkyv::api::low::deserialize::<_, Failure>(archive).map_err(|_| ())?;
        Ok(())
    }

    pub fn load_config_bytes_to_flash(&self, bytes: ArrayVec<u8, CONFIG_SIZE>) -> Result<(), ()> {
        let size = bytes.len();
        if size > CONFIG_SIZE {
            error!("config bytes length prefix too long {}", size);
            return Err(());
        }

        self.check_valid_config(&bytes)?;

        // let mut data = ArrayVec::new();
        // TODO: write size + body to data

        // Safety: This byte range is known to be valid flash memory on this device
        // unsafe {
        //     for i in 0..4 {
        //         let address = (RP2040_FLASH_OFFSET + CONFIG_OFFSET + i) as *mut u8;
        //         core::ptr::write_volatile::<u8>(address) as u32) << ((3 - i) * 8);
        //     }

        //     let size = size as usize;
        //     for i in 0..size {
        //         let address = (RP2040_FLASH_OFFSET + CONFIG_OFFSET + 4 + i) as *mut u8;
        //         data.push(core::ptr::read_volatile::<u8>(address));
        //     }
        // }
        Ok(())
    }
}
