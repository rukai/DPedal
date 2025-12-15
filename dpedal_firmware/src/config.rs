use arrayvec::ArrayVec;
use defmt::error;
use dpedal_config::{ArchivedConfig, CONFIG_OFFSET, CONFIG_SIZE, Config, RP2040_FLASH_SIZE};
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

    pub fn load(&mut self) -> Result<Config, ()> {
        let bytes = self.load_config_bytes_from_flash()?;
        let archive = rkyv::api::low::access::<ArchivedConfig, Failure>(&bytes).map_err(|_| ())?;
        rkyv::api::low::deserialize::<_, Failure>(archive).map_err(|_| ())
    }

    pub fn load_config_bytes_from_flash(&mut self) -> Result<Align<ArrayVec<u8, CONFIG_SIZE>>, ()> {
        // TODO: store in heap instead, apparently only 2kb of stack o.0
        let mut bytes = [0u8; CONFIG_SIZE];
        self.flash
            .blocking_read(CONFIG_OFFSET as u32, &mut bytes)
            .unwrap();
        let size = u32::from_be_bytes(bytes[..4].try_into().unwrap());

        let size = size as usize;
        if size > CONFIG_SIZE - 4 {
            error!("config bytes length prefix too long {}", size);
            return Err(());
        }

        Ok(Align(ArrayVec::from_iter(
            bytes[4..4 + size].iter().cloned(),
        )))
    }

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
        //         let address = (RP2040_FLASH_OFFSET + CONFIG_OFFSET + + i) as *mut u8;
        //         data.push(core::ptr::read_volatile::<u8>(address));
        //     }
        // }
        Ok(())
    }
}
