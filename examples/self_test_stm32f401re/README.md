# IIS2MDC Magnetometer Self-Test Example on STM32F401RE Nucleo-64

This example demonstrates how to perform a **self-test procedure** on the IIS2MDC 3-axis magnetometer using the STM32F401RE Nucleo-64 board. The firmware configures the sensor, collects magnetic field data before and during self-test mode, and checks if the self-test response is within the expected range for each axis. Results are printed over UART.

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
- **Sensor:** Initializes the IIS2MDC driver and checks the device ID.

### Self-Test Procedure

- **Normal Mode Sampling:**
  - Restores the sensor to default configuration.
  - Enables block data update, temperature compensation, and sets continuous mode at 100 Hz.
  - Waits for sensor stabilization and flushes old samples.
  - Collects 50 samples of magnetic field data for each axis and computes the average (media).
- **Self-Test Mode Sampling:**
  - Enables self-test mode and waits for stabilization.
  - Flushes old samples.
  - Collects 50 samples of magnetic field data for each axis and computes the average (mediast).
- **Self-Test Evaluation:**
  - For each axis, computes the absolute difference between self-test and normal averages.
  - Checks if the difference is within the expected range (15 mG to 500 mG).
  - Prints "PASSED" or "FAILED" for each axis over UART.
- **Cleanup:**
  - Disables self-test mode and powers down the sensor.

### Helper Function

- **flush_samples:** Reads and discards a sample if data is ready, ensuring only fresh data is used for measurements.

---

## Usage

1. Connect the IIS2MDC sensor to the STM32F401RE Nucleo board via I2C1 (PB8/PB9).
2. Build and flash the firmware onto the STM32F401RE board.
3. Open a serial terminal at 115200 baud on the USART2 TX line (PA2).
4. Observe the self-test results ("PASSED"/"FAILED") for each axis printed in the terminal.
5. The self-test procedure repeats in a loop for continuous monitoring.

---

## Notes

- The self-test checks if the sensor's response is within the specified range for each axis, as recommended in the IIS2MDC datasheet.
- Ensure proper pull-up resistors are present on the I2C lines if not provided by the sensor board.
- The code uses the [`iis2mdc`](https://crates.io/crates/iis2mdc) Rust driver and [`stm32f4xx-hal`](https://docs.rs/stm32f4xx-hal) HAL crate.
- Error handling is minimal for clarity; production code should handle errors more robustly.
- The self-test is performed in a loop for demonstration purposes; in a real application, it may be run only at startup or on demand.

---

## References

- [STM32F401RE Nucleo-64 Board](https://www.st.com/en/evaluation-tools/nucleo-f401re.html)
- [IIS2MDC Datasheet](https://www.st.com/resource/en/datasheet/iis2mdc.pdf)
- [stm32f4xx-hal Rust crate](https://docs.rs/stm32f4xx-hal)

---

*This README explains how to perform and evaluate the self-test procedure of the IIS2MDC magnetometer on the STM32F401RE Nucleo-64 board using Rust and the stm32f4xx-hal crate.*