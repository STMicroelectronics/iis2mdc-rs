# iis2mdc-rs
[![Crates.io][crates-badge]][crates-url]
[![BSD 3-Clause licensed][bsd-badge]][bsd-url]

[crates-badge]: https://img.shields.io/crates/v/iis2mdc-rs
[crates-url]: https://crates.io/crates/iis2mdc-rs
[bsd-badge]: https://img.shields.io/crates/l/iis2mdc-rs
[bsd-url]: https://opensource.org/licenses/BSD-3-Clause

This crate provides a platform-agnostic driver for the ST IIS2MDC digital magnetic sensor, supporting both I2C and SPI communication interfaces.

## Sensor Overview

The IIS2MDC is a high-accuracy, ultra-low-power
3-axis digital magnetic sensor.
The IIS2MDC has a magnetic field dynamic range
up to ±50 gauss.
The IIS2MDC includes an I2C serial bus interface
that supports standard, fast mode, fast mode
plus, and high-speed (100 kHz, 400 kHz,
1 MHz, and 3.4 MHz) and an SPI serial standard
interface.
The device can be configured to generate an
interrupt signal for magnetic field detection.
The IIS2MDC is available in a plastic land grid
array package (LGA) and is guaranteed to
operate over an extended temperature range
from -40 °C to +85 °C. 

For more info, please visit the device page at [ST IIS2MDC](https://www.st.com/en/mems-and-sensors/iis2mdc.html).

## Installation

Add the driver to your `Cargo.toml` dependencies:

```toml
[dependencies]
iis2mdc-rs = "0.1.0"
```

Or, add it directly from the terminal:

```sh
cargo add iis2mdc-rs
```

## Usage

Include the crate and its prelude
```rust
use iis2mdc_rs as iis2mdc;
use iis2mdc::*;
use iis2mdc::prelude::*;
```

### Create an instance

Create an instance of the driver with the `new_<bus>` associated function, by passing an I2C (`embedded_hal::i2c::I2c`) instance and I2C address, or an SPI (`embedded_hal::spi::SpiDevice`) instance, along with a timing peripheral.

An example with I2C:

```rust
let mut sensor = Iis2mdc::new_i2c(i2c, I2CAddress::I2cAdd, delay);
```

### Check "Who Am I" Register

This step ensures correct communication with the sensor. It returns a unique ID to verify the sensor's identity.

```rust
let whoami = sensor.device_id_get().unwrap();
if whoami != ID {
    panic!("Invalid sensor ID");
}
```

### Configure

See details in specific examples; the following are common api calls:

```rust
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
```

## License

Distributed under the BSD-3 Clause license.

More Information: [http://www.st.com](http://st.com/MEMS).

**Copyright (C) 2025 STMicroelectronics**