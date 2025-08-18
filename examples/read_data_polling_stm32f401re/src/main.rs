#![no_main]
#![no_std]

use core::fmt::Write;

use iis2mdc_rs::prelude::*;
use iis2mdc_rs::Iis2mdc;
use panic_itm as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    hal::delay::DelayNs,
    i2c::{DutyCycle, I2c, Mode},
    pac,
    prelude::*,
    serial::Config,
};

const PROPERTY_ENABLED: u8 = 1;

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
                tx,
                "Magnetic field [mG]: {:4.2} {:4.2} {:4.2}",
                magnetic_mg[0], magnetic_mg[1], magnetic_mg[2]
            )
            .unwrap();
            // Read temperature data
            temperature_degc =
                iis2mdc_rs::from_lsb_to_celsius(sensor.temperature_raw_get().unwrap());
            writeln!(tx, "Temperature [degC]: {:6.2}", temperature_degc).unwrap();

            sensor.tim.delay_ms(1000);
        }
    }
}
