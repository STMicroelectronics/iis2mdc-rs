#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::khz;
use embassy_stm32::usart::{self, DataBits, Parity, UartTx};
use embassy_time::Delay;
use embedded_hal::delay::DelayNs;
use heapless::String;
use iis2mdc_rs::prelude::*;
use iis2mdc_rs::Iis2mdc;

use {defmt_rtt as _, panic_probe as _};

#[defmt::panic_handler]
fn panic() -> ! {
    core::panic!("panic via `defmt::panic!")
}

const PROPERTY_ENABLED: u8 = 1;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut usart_cfg = usart::Config::default();
    usart_cfg.baudrate = 115200;
    usart_cfg.data_bits = DataBits::DataBits8;
    usart_cfg.parity = Parity::ParityNone;

    let mut tx = UartTx::new_blocking(p.USART2, p.PA2, usart_cfg).unwrap();

    let i2c = I2c::new_blocking(p.I2C1, p.PB8, p.PB9, khz(100), Default::default());

    let mut delay = Delay;
    let mut msg = String::<64>::new();

    delay.delay_ms(10);

    let mut sensor = Iis2mdc::new_i2c(i2c, iis2mdc_rs::I2CAddress::I2cAdd, delay);
    match sensor.device_id_get() {
        Ok(value) => {
            if value != iis2mdc_rs::IIS2MDC_ID {
                panic!("Invalid sensor ID")
            }
        }
        Err(e) => {
            writeln!(&mut msg, "An error occured while reading sensor ID: {e:?}").unwrap();
            tx.blocking_write(msg.as_bytes()).unwrap();
            msg.clear();
        }
    }

    // Restore default configuration
    sensor.reset_set(PROPERTY_ENABLED).unwrap();
    loop {
        if sensor.reset_get().unwrap() == 0 {
            break;
        }
    }

    // Enable block data update
    sensor.block_data_update_set(PROPERTY_ENABLED).unwrap();
    // Set output data rate
    sensor.data_rate_set(Odr::_10hz).unwrap();
    // Set / Reset sensor mode
    sensor
        .set_rst_mode_set(SetRst::SensOffCancEveryOdr)
        .unwrap();

    // Enable temperature compensation
    sensor.offset_temp_comp_set(PROPERTY_ENABLED).unwrap();
    // Set device in continuous mode
    sensor.operating_mode_set(Md::ContinuousMode).unwrap();
    // Power up and wait for 20ms for stable output
    sensor.tim.delay_ms(20);

    let mut magnetic_mg = [0.0_f32; 3];
    let mut temperature_degc;

    // Read samples in polling mode (no int)
    loop {
        let drdy = sensor.mag_data_ready_get().unwrap();
        if drdy == 1 {
            // Read magnetic field data
            let raw_magnetic = sensor.magnetic_raw_get().unwrap();
            (0..3)
                .for_each(|i| magnetic_mg[i] = iis2mdc_rs::from_lsb_to_mgauss(raw_magnetic[i]));
            writeln!(
                &mut msg,
                "Magnetic field [mG]: {:4.2} {:4.2} {:4.2}",
                magnetic_mg[0], magnetic_mg[1], magnetic_mg[2]
            )
            .unwrap();
            tx.blocking_write(msg.as_bytes()).unwrap();
            msg.clear();

            // Read temperature data
            temperature_degc =
                iis2mdc_rs::from_lsb_to_celsius(sensor.temperature_raw_get().unwrap());
            writeln!(&mut msg, "Temperature [degC]: {:6.2}", temperature_degc).unwrap();
            tx.blocking_write(msg.as_bytes()).unwrap();
            msg.clear();

            sensor.tim.delay_ms(1000);
        }
    }
}
