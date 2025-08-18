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
use st_mems_bus::BusOperation;

use {defmt_rtt as _, panic_probe as _};

#[defmt::panic_handler]
fn panic() -> ! {
    core::panic!("panic via `defmt::panic!")
}

const PROPERTY_ENABLED: u8 = 1;
const PROPERTY_DISABLED: u8 = 0;

const SELF_TEST_SAMPLES: usize = 50;

const ST_MIN_POS: f32 = 15.;
const ST_MAX_POS: f32 = 500.;

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

    let mut magnetic_m_g: [[f32; 3]; SELF_TEST_SAMPLES] = [[0.; 3]; 50];

    let mut media: [f32; 3] = [0.; 3];
    let mut mediast: [f32; 3] = [0.; 3];

    loop {
        // Restore default configuration
        sensor.reset_set(PROPERTY_ENABLED).unwrap();
        loop {
            if sensor.reset_get().unwrap() == 0 {
                break;
            }
        }

        sensor.block_data_update_set(PROPERTY_ENABLED).unwrap();
        // Set / Reset sensor mode
        sensor
            .set_rst_mode_set(SetRst::SensOffCancEveryOdr)
            .unwrap();
        // Enable temperature compensation
        sensor.offset_temp_comp_set(PROPERTY_ENABLED).unwrap();
        // Set device in continuous mode
        sensor.operating_mode_set(Md::ContinuousMode).unwrap();
        // Set output data rate to 100 Hz
        sensor.data_rate_set(Odr::_100hz).unwrap();
        // Power up and wait for 20ms for stable output
        sensor.tim.delay_ms(20);
        // Flush old samples
        flush_samples(&mut sensor).unwrap();

        let mut i = 0;
        loop {
            let rdy = sensor.mag_data_ready_get().unwrap();
            if rdy == 1 {
                let raw_data = sensor.magnetic_raw_get().unwrap();
                (0..3).for_each(|axis| {
                    magnetic_m_g[i][axis] = iis2mdc_rs::from_lsb_to_mgauss(raw_data[axis]);
                });
                i += 1;
            }
            if i >= SELF_TEST_SAMPLES {
                break;
            }
        }

        (0..3).for_each(|i| {
            (0..SELF_TEST_SAMPLES).for_each(|j| {
                media[i] += magnetic_m_g[j][i];
            });
            media[i] /= SELF_TEST_SAMPLES as f32;
        });

        // Enable self test mode
        sensor.self_test_set(PROPERTY_ENABLED).unwrap();
        sensor.tim.delay_ms(60);
        // Flush old samples
        flush_samples(&mut sensor).unwrap();
        i = 0;
        loop {
            let rdy = sensor.mag_data_ready_get().unwrap();
            if rdy == 1 {
                let raw_data = sensor.magnetic_raw_get().unwrap();
                (0..3).for_each(|axis| {
                    magnetic_m_g[i][axis] = iis2mdc_rs::from_lsb_to_mgauss(raw_data[axis]);
                });
                i += 1;
            }
            if i >= SELF_TEST_SAMPLES {
                break;
            }
        }

        (0..3).for_each(|i| {
            (0..SELF_TEST_SAMPLES).for_each(|j| {
                mediast[i] += magnetic_m_g[j][i];
            });
            mediast[i] /= SELF_TEST_SAMPLES as f32;
        });

        // Check for all axis self test value range
        let mut passed = [false; 3];
        (0..3).for_each(|i| {
            let diff = (mediast[i] - media[i]).abs();
            if (ST_MIN_POS..=ST_MAX_POS).contains(&diff) {
                passed[i] = true;
            }

            writeln!(
                &mut msg,
                "{}: |{}| <= |{}| <= |{}| {}",
                i,
                ST_MIN_POS,
                diff,
                ST_MAX_POS,
                if passed[i] { "PASSED" } else { "FAILED" }
            )
            .unwrap();
            tx.blocking_write(msg.as_bytes()).unwrap();
            msg.clear();
        });

        // Disable self test mode
        sensor.operating_mode_set(Md::PowerDown).unwrap();
        sensor.self_test_set(PROPERTY_DISABLED).unwrap();
    }
}

fn flush_samples<B, T>(s: &mut Iis2mdc<B, T>) -> Result<(), iis2mdc_rs::Error<B::Error>>
where
    B: BusOperation,
    T: DelayNs,
{
    let rdy = s.mag_data_ready_get()?;
    if rdy == 1 {
        let _ = s.magnetic_raw_get()?;
    }
    Ok(())
}
