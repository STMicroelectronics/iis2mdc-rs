use defmt::info;
use maybe_async::maybe_async;
use crate::*;
use iis2mdc::prelude::*;
use iis2mdc::*;

const PROPERTY_ENABLED: u8 = 1;
const PROPERTY_DISABLED: u8 = 0;

const SELF_TEST_SAMPLES: usize = 50;

const ST_MIN_POS: f32 = 15.;
const ST_MAX_POS: f32 = 500.;

#[maybe_async]
pub async fn run<B, D, L>(bus: B, mut tx: L, mut delay: D, _irq: ()) -> !
where
    B: BusOperation,
    D: DelayNs + Clone,
    L: embedded_io::Write
{


    info!("Configuring the sensor");
    let mut sensor = Iis2mdc::from_bus(bus, delay.clone());

    // boot time
    delay.delay_ms(10).await;

    // Check device ID
    let id = sensor.device_id_get().await.unwrap();
    info!("Device ID: {:x}", id);
    if id != IIS2MDC_ID {
        info!("Unexpected device ID: {:x}", id);
        writeln!(tx, "Unexpected device ID: {:x}", id).unwrap();
        loop {}
    }

    let mut magnetic_m_g: [[f32; 3]; SELF_TEST_SAMPLES] = [[0.; 3]; 50];

    let mut media: [f32; 3] = [0.; 3];
    let mut mediast: [f32; 3] = [0.; 3];

    loop {
        // Restore default configuration
        sensor.reset_set(PROPERTY_ENABLED).await.unwrap();
        loop {
            if sensor.reset_get().await.unwrap() == 0 {
                break;
            }
        }

        sensor.block_data_update_set(PROPERTY_ENABLED).await.unwrap();
        // Set / Reset sensor mode
        sensor
            .set_rst_mode_set(SetRst::SensOffCancEveryOdr)
            .await.unwrap();
        // Enable temperature compensation
        sensor.offset_temp_comp_set(PROPERTY_ENABLED).await.unwrap();
        // Set device in continuous mode
        sensor.operating_mode_set(Md::ContinuousMode).await.unwrap();
        // Set output data rate to 100 Hz
        sensor.data_rate_set(Odr::_100hz).await.unwrap();
        // Power up and wait for 20ms for stable output
        delay.delay_ms(20).await;
        // Flush old samples
        flush_samples(&mut sensor).await.unwrap();

        let mut i = 0;
        loop {
            let rdy = sensor.mag_data_ready_get().await.unwrap();
            if rdy == 1 {
                let raw_data = sensor.magnetic_raw_get().await.unwrap();
                (0..3).for_each(|axis| {
                    magnetic_m_g[i][axis] = from_lsb_to_mgauss(raw_data[axis]);
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
        sensor.self_test_set(PROPERTY_ENABLED).await.unwrap();
        delay.delay_ms(60).await;
        // Flush old samples
        flush_samples(&mut sensor).await.unwrap();
        i = 0;
        loop {
            let rdy = sensor.mag_data_ready_get().await.unwrap();
            if rdy == 1 {
                let raw_data = sensor.magnetic_raw_get().await.unwrap();
                (0..3).for_each(|axis| {
                    magnetic_m_g[i][axis] = from_lsb_to_mgauss(raw_data[axis]);
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
        sensor.operating_mode_set(Md::PowerDown).await.unwrap();
        sensor.self_test_set(PROPERTY_DISABLED).await.unwrap();
    }
}

#[maybe_async]
async fn flush_samples<B, T>(s: &mut Iis2mdc<B, T, OnState>) -> Result<(), iis2mdc::Error<B::Error>>
where
    B: BusOperation,
    T: DelayNs,
{
    let rdy = s.mag_data_ready_get().await?;
    if rdy == 1 {
        let _ = s.magnetic_raw_get().await?;
    }
    Ok(())
}
