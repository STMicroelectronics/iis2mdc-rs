# IIS2MDC Magnetometer Example on STM32F401RE Nucleo-64

This example demonstrates how to interface the **IIS2MDC 3-axis magnetometer** with an STM32F401RE Nucleo-64 board using the I2C interface. The firmware initializes the sensor, configures it for continuous measurement, and periodically reads and prints the magnetic field and temperature data over UART.

---

## Hardware Setup

- **Microcontroller Board:** STM32F401RE Nucleo-64
- **Sensor:** IIS2MDC Magnetometer
- **Communication Interface:** I2C1 at 100 kHz (Standard Mode)
- **UART:** USART2 for serial output at 115200 baud

### Default Pin Configuration

| Signal       | STM32F401RE Pin | Description                    |
|--------------|-----------------|-------------------------------|
| I2C1_SCL     | PB8             | I2C clock line (open-drain)   |
| I2C1_SDA     | PB9             | I2C data line (open-drain)    |
| USART2_TX    | PA2             | UART transmit for debug output|

The IIS2MDC sensor is connected to the STM32F401RE via I2C1 on pins PB8 (SCL) and PB9 (SDA). UART output is routed through PA2.

---

## Code Description

### Initialization

- **Clocks:** Configures the system clock using an 8 MHz external oscillator.
- **Delay:** Sets up a delay provider using the SysTick timer.
- **GPIO:** Splits GPIOA and GPIOB for pin configuration.
- **I2C:** Initializes I2C1 in standard mode (100 kHz) on PB8 (SCL) and PB9 (SDA).
- **UART:** Configures USART2 for serial output at 115200 baud on PA2 (TX).

### Sensor Configuration

- **Device ID Check:** Reads and verifies the IIS2MDC device ID.
- **Reset:** Restores the sensor to its default configuration and waits for completion.
- **Block Data Update:** Enables block data update to ensure data consistency.
- **Output Data Rate:** Sets the output data rate to 10 Hz.
- **Set/Reset Mode:** Configures the set/reset sensor mode for offset cancellation.
- **Temperature Compensation:** Enables temperature compensation for improved accuracy.
- **Operating Mode:** Sets the sensor to continuous measurement mode.
- **Stabilization Delay:** Waits 20 ms for the sensor output to stabilize.

### Data Acquisition Loop

- **Polling:** Continuously checks if new magnetic data is available.
- **Magnetic Field Read:** Reads raw magnetic data, converts it to milliGauss, and prints it over UART.
- **Temperature Read:** Reads raw temperature data, converts it to degrees Celsius, and prints it over UART.
- **Delay:** Waits 1 second between readings.

---

## Usage

1. Connect the IIS2MDC sensor to the STM32F401RE Nucleo board via I2C1 (PB8/PB9).
2. Build and flash the firmware onto the STM32F401RE board.
3. Open a serial terminal at 115200 baud on the USART2 TX line (PA2).
4. Observe magnetic field (in mG) and temperature (in °C) readings printed every second.

---

## Notes

- The example uses polling mode for data acquisition; no interrupts are required.
- Ensure proper pull-up resistors are present on the I2C lines if not provided by the sensor board.
- The code uses the [`iis2mdc`](https://crates.io/crates/iis2mdc) Rust driver and [`stm32f4xx-hal`](https://docs.rs/stm32f4xx-hal) HAL crate.
- Error handling is minimal for clarity; production code should handle errors more robustly.

---

## References

- [STM32F401RE Nucleo-64 Board](https://www.st.com/en/evaluation-tools/nucleo-f401re.html)
- [IIS2MDC Datasheet](https://www.st.com/resource/en/datasheet/iis2mdc.pdf)
- [stm32f4xx-hal Rust crate](https://docs.rs/stm32f4xx-hal)

---

*This README explains how to use the IIS2MDC magnetometer with the STM32F401RE Nucleo-64 board using Rust and the stm32f4xx-hal crate.*