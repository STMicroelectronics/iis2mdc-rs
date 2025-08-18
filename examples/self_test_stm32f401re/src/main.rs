#![no_main]
#![no_std]

use core::fmt::Write;

use iis2mdc_rs::prelude::*;
use iis2mdc_rs::Iis2mdc;
use panic_itm as _;

use cortex_m_rt::entry;
use st_mems_bus::BusOperation;
use stm32f4xx_hal::{
    hal::delay::DelayNs,
    i2c::{DutyCycle, I2c, Mode},
    pac,
    prelude::*,
    serial::Config,
};

const PROPERTY_ENABLED: u8 = 1;
const PROPERTY_DISABLED: u8 = 0;

const SELF_TEST_SAMPLES: usize = 50;

const ST_MIN_POS: f32 = 15.;
const ST_MAX_POS: f32 = 500.;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(48.MHz()).freeze();

    let mut delay = cp.SYST.delay(&clocks);

    let gpiob = dp.GPIOB.split();
    let gpioa = dp.GPIOA.split();

    let scl = gpiob.pb8;
    let sda = gpiob.pb9;

    let i2c = I2c::new(
        dp.I2C1,
        (scl, sda),
        Mode::Fast {
            frequency: 400.kHz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        &clocks,
    );

    let tx_pin = gpioa.pa2.into_alternate();
    let mut tx = dp
        .USART2
        .tx(
            tx_pin,
            Config::default()
                .baudrate(115200.bps())
                .wordlength_8()
                .parity_none(),
            &clocks,
        )
        .unwrap();

    delay.delay_ms(10);

    let mut sensor = Iis2mdc::new_i2c(i2c, iis2mdc_rs::I2CAddress::I2cAdd, delay);
    match sensor.device_id_get() {
        Ok(value) => {
            if value != iis2mdc_rs::IIS2MDC_ID {
                panic!("Invalid sensor ID")
            }
        }
        Err(e) => writeln!(tx, "An error occured while reading sensor ID: {e:?}").unwrap(),
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
                tx,
                "{}: |{}| <= |{}| <= |{}| {}",
                i,
                ST_MIN_POS,
                diff,
                ST_MAX_POS,
                if passed[i] { "PASSED" } else { "FAILED" }
            )
            .unwrap();
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
