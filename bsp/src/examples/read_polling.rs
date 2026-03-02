use defmt::info;
use maybe_async::maybe_async;
use crate::*;

#[maybe_async]
pub async fn run<B, D, L>(bus: B, mut tx: L, mut delay: D, _irq: ()) -> !
where
    B: BusOperation,
    D: DelayNs + Clone,
    L: embedded_io::Write
{
    use iis2mdc::prelude::*;
    use iis2mdc::*;

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

    // Restore default configuration
    sensor.reset_set(1).await.unwrap();
    loop {
        if sensor.reset_get().await.unwrap() == 0 {
            break;
        }
    }

    // Enable block data update
    sensor.block_data_update_set(1).await.unwrap();
    // Set output data rate
    sensor.data_rate_set(Odr::_10hz).await.unwrap();
    // Set / Reset sensor mode
    sensor
        .set_rst_mode_set(SetRst::SensOffCancEveryOdr)
        .await.unwrap();

    // Enable temperature compensation
    sensor.offset_temp_comp_set(1).await.unwrap();
    // Set device in continuous mode
    sensor.operating_mode_set(Md::ContinuousMode).await.unwrap();
    // Power up and wait for 20ms for stable output
    delay.delay_ms(20).await;

    let mut magnetic_mg = [0.0_f32; 3];
    let mut temperature_degc;

    // Read samples in polling mode (no int)
    loop {
        let drdy = sensor.mag_data_ready_get().await.unwrap();
        if drdy == 1 {
            // Read magnetic field data
            let raw_magnetic = sensor.magnetic_raw_get().await.unwrap();
            (0..3)
                .for_each(|i| magnetic_mg[i] = from_lsb_to_mgauss(raw_magnetic[i]));
            writeln!(
                tx,
                "Magnetic field [mG]: {:4.2} {:4.2} {:4.2}",
                magnetic_mg[0], magnetic_mg[1], magnetic_mg[2]
            )
            .unwrap();
            // Read temperature data
            temperature_degc =
                from_lsb_to_celsius(sensor.temperature_raw_get().await.unwrap());
            writeln!(tx, "Temperature [degC]: {:6.2}", temperature_degc).unwrap();

            delay.delay_ms(1000).await;
        }
    }
}
