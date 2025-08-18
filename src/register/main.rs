use crate::BusOperation;
use crate::{Error, Iis2mdc};
use bitfield_struct::bitfield;
use derive_more::TryFrom;
use embedded_hal::delay::DelayNs;

use st_mem_bank_macro::{named_register, register};

/// Represents the register addresses for device configuration and data retrieval.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum Reg {
    /// Address for the low byte of the X-axis offset register.
    OffsetXRegL = 0x45,
    /// Address for the high byte of the X-axis offset register.
    OffsetXRegH = 0x46,
    /// Address for the low byte of the Y-axis offset register.
    OffsetYRegL = 0x47,
    /// Address for the high byte of the Y-axis offset register.
    OffsetYRegH = 0x48,
    /// Address for the low byte of the Z-axis offset register.
    OffsetZRegL = 0x49,
    /// Address for the high byte of the Z-axis offset register.
    OffsetZRegH = 0x4A,
    /// Address for the Who am I register, used to identify the device.
    WhoAmI = 0x4F,
    /// Address for the configuration register A.
    CfgRegA = 0x60,
    /// Address for the configuration register B.
    CfgRegB = 0x61,
    /// Address for the configuration register C.
    CfgRegC = 0x62,
    /// Address for the interrupt control register.
    IntCtrlReg = 0x63,
    /// Address for the interrupt source register.
    IntSourceReg = 0x64,
    /// Address for the low byte of the interrupt threshold value register.
    IntThsLReg = 0x65,
    /// Address for the high byte of the interrupt threshold value register.
    IntThsHReg = 0x66,
    /// Address for the status register, providing device status information.
    StatusReg = 0x67,
    /// Address for the low byte of the X-axis output register.
    OutxLReg = 0x68,
    /// Address for the high byte of the X-axis output register.
    OutxHReg = 0x69,
    /// Address for the low byte of the Y-axis output register.
    OutyLReg = 0x6A,
    /// Address for the high byte of the Y-axis output register.
    OutyHReg = 0x6B,
    /// Address for the low byte of the Z-axis output register.
    OutzLReg = 0x6C,
    /// Address for the high byte of the Z-axis output register.
    OutzHReg = 0x6D,
    /// Address for the low byte of the temperature output register.
    TempOutLReg = 0x6E,
    /// Address for the high byte of the temperature output register.
    TempOutHReg = 0x6F,
}

/// Configuration register A.
///
/// The configuration register is used to configure the output data rate and the measurement configuration.
///
/// The bit order for this struct can be configured using the `bit_order_msb` feature:
/// * `Msb`: Most significant bit first.
/// * `Lsb`: Least significant bit first (default).
#[register(address = Reg::CfgRegA, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u8, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u8, order = Lsb))]
pub struct CfgRegA {
    /// This bits select the mode of operation of the device.
    #[bits(2, default = 0b11)]
    pub md: u8,
    /// Output data rate configuration.
    #[bits(2)]
    pub odr: u8,
    /// Enables low-power mode.
    #[bits(1)]
    pub lp: u8,
    /// When this bit is set, the configuration registers and user registers are reset. Flash
    /// register keep their values.
    #[bits(1)]
    pub soft_rst: u8,
    /// Reboot magnetometer memory content.
    #[bits(1)]
    pub reboot: u8,
    /// Enables the magnetometer temperature compensation.
    #[bits(1)]
    pub comp_temp_en: u8,
}

/// Configuration register B.
///
/// The configuration register is used to configure the output data rate and the measurement configuration.
///
/// The bit order for this struct can be configured using the `bit_order_msb` feature:
/// * `Msb`: Most significant bit first.
/// * `Lsb`: Least significant bit first (default).
#[register(address = Reg::CfgRegB, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u8, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u8, order = Lsb))]
pub struct CfgRegB {
    /// Enables low-pass filter.
    #[bits(1)]
    pub lpf: u8,
    /// `OFF_CANC` + `Set_FREQ`.
    /// * `OFF_CANC`: Enables offset calibration.
    /// * `Set_FREQ`: Selects the frequency of the set pulse.
    #[bits(2)]
    pub set_rst: u8,
    /// If `1`, the interrupt block recognition checks data after the hard-iron correction to
    /// discover the interrupt.
    #[bits(1)]
    pub int_on_dataoff: u8,
    /// Enables offset cancellation in single measurement mode. The `OFF_CANC` bit must be set to
    /// `1` when enabling offset cancellation in single measurement mode.
    #[bits(1)]
    pub off_canc_one_shot: u8,
    #[bits(3, access = RO)]
    not_used_01: u8,
}

/// Configuration register C.
///
/// The configuration register is used to configure the output data rate and the measurement configuration.
///
/// The bit order for this struct can be configured using the `bit_order_msb` feature:
/// * `Msb`: Most significant bit first.
/// * `Lsb`: Least significant bit first (default).
#[register(address = Reg::CfgRegC, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u8, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u8, order = Lsb))]
pub struct CfgRegC {
    /// If `1`, the data-ready signal is driven on the INT/DRDY pin. The INT/DRDY pin is configured
    /// in push-pull output mode.
    #[bits(1)]
    pub drdy_on_pin: u8,
    /// If `1`, the self-test is enabled.
    #[bits(1)]
    pub self_test: u8,
    /// Must be set to `0`
    #[bits(1, access = RO, default = 0x0)]
    not_used_01: u8,
    /// If `1`, an inversion of the low and high parts of the data occurs.
    #[bits(1)]
    pub ble: u8,
    /// If enabled, reading of incorrect data is avoided when the user reads asynchronously. In fact
    /// if the read request arrives during an update of the output data, a latch is possible, reading
    /// incoherent high and low parts of the same register. Only one part is updated and the other
    /// one remains old
    #[bits(1)]
    pub bdu: u8,
    /// If `1`, the I2C interface is inhibited. Only the SPI interface can be used.
    #[bits(1)]
    pub i2c_dis: u8,
    /// If `1`, the INTERRUPT signal is driver on the INT/DRDY pin. The INT/DRDY pin is configured
    /// in push-pull output mode.
    #[bits(1)]
    pub int_on_pin: u8,
    #[bits(1, access = RO)]
    not_used_02: u8,
}

/// Interrupt control register.
///
/// The interrupt control register is used to enable and to configure the interrupt recognition.
///
/// The bit order for this struct can be configured using the `bit_order_msb` feature:
/// * `Msb`: Most significant bit first.
/// * `Lsb`: Least significant bit first (default).
#[register(address = Reg::IntCtrlReg, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u8, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u8, order = Lsb))]
pub struct IntCtrlReg {
    /// Interrupt enable. When set, enables the interrupt generation.
    #[bits(1)]
    pub ien: u8,
    /// Controls whether the INT bit ([`IntSourceReg`]), is latched or pulsed. Once latched, INT remains in the same
    /// state until `IntSourceReg` is read.
    #[bits(1)]
    pub iel: u8,
    /// Controls the polarity of the INT bit ([`IntSourceReg`]) when an interrupt occurs.
    #[bits(1)]
    pub iea: u8,
    /// This bits must be set to `0`.
    #[bits(2, access = RO, default = 0x00)]
    not_used_01: u8,
    /// Enables the interrupt detection for the Z-axis.
    #[bits(1, default = 0x1)]
    pub zien: u8,
    /// Enables the interrupt detection for the Y-axis.
    #[bits(1, default = 0x1)]
    pub yien: u8,
    /// Enables the interrupt detection for the X-axis.
    #[bits(1, default = 0x1)]
    pub xien: u8,
}

/// Interrupt source register (read only).
///
/// When interrupt latched is selected, reading this register resets all the bits in this register.
///
/// The bit order for this struct can be configured using the `bit_order_msb` feature:
/// * `Msb`: Most significant bit first.
/// * `Lsb`: Least significant bit first (default).
#[register(address = Reg::IntSourceReg, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u8, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u8, order = Lsb))]
pub struct IntSourceReg {
    /// This bit signals when the interrupt event occurs.
    #[bits(1, access = RO)]
    pub int: u8,
    /// MROI flag generation is always enabled. This flag is reset by reading `IntSourceReg`.
    #[bits(1, access = RO)]
    pub mroi: u8,
    /// Z-axis value exceeds the threshold negative side.
    #[bits(1, access = RO)]
    pub n_th_s_z: u8,
    /// Y-axis value exceeds the threshold negative side.
    #[bits(1, access = RO)]
    pub n_th_s_y: u8,
    #[bits(1, access = RO)]
    /// X-axis value exceeds the threshold negative side.
    pub n_th_s_x: u8,
    #[bits(1, access = RO)]
    /// Z-axis value exceeds the threshold positive side.
    pub p_th_s_z: u8,
    /// Y-axis value exceeds the threshold positive side.
    #[bits(1, access = RO)]
    pub p_th_s_y: u8,
    /// X-axis value exceeds the threshold positive side.
    #[bits(1, access = RO)]
    pub p_th_s_x: u8,
}

/// Status register (read only).
///
/// This register is used to indicate device status.
///
/// The bit order for this struct can be configured using the `bit_order_msb` feature:
/// * `Msb`: Most significant bit first.
/// * `Lsb`: Least significant bit first (default).
#[register(address = Reg::StatusReg, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u8, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u8, order = Lsb))]
pub struct StatusReg {
    /// X-axis new data available.
    #[bits(1, access = RO)]
    pub xda: u8,
    /// Y-axis new data available.
    #[bits(1, access = RO)]
    pub yda: u8,
    /// Z-axis new data available.
    #[bits(1, access = RO)]
    pub zda: u8,
    /// X-, Y-, Z-axis new data available.
    #[bits(1, access = RO)]
    pub zyxda: u8,
    #[bits(1, access = RO)]
    /// X-axis data overrun.
    pub xor: u8,
    /// Y-axis data overrun.
    #[bits(1, access = RO)]
    pub yor: u8,
    /// Z-axis data overrun.
    #[bits(1, access = RO)]
    pub zor: u8,
    /// X-, Y-, Z-axis data overrun.
    #[bits(1, access = RO)]
    pub zyxor: u8,
}

/// Offset registers for the X, Y, and Z axes.
///
/// This register group holds the hard-iron offset values used to compensate the magnetic sensor readings.
/// The offsets are represented as a three-element array of 16-bit signed integers, corresponding to the X, Y, and Z axes respectively.
#[named_register(address = Reg::OffsetXRegL, access_type = Iis2mdc, generics = 2)]
pub struct OffsetXYZ {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// Output registers for the X, Y, and Z axes.
///
/// This register group contains the raw magnetic output data from the sensor.
/// The data is represented as a three-element array of 16-bit signed integers, corresponding to the X, Y, and Z axes respectively.
#[named_register(address = Reg::OutxLReg, access_type = Iis2mdc, generics = 2)]
pub struct OutXYZ {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// Temperature output register (read-only).
///
/// This register holds the raw temperature measurement from the sensor as a 16-bit signed integer.
/// The bit order can be configured via the `bit_order_msb` feature flag.
/// The temperature value is accessible through the `temp_out` field.
#[register(address = Reg::TempOutLReg, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u16, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u16, order = Lsb))]
pub struct TempOutReg {
    /// Raw temperature output value (16 bits, read-only, signed).
    #[bits(16, access = RO)]
    pub temp_out: i16,
}

/// Interrupt threshold register.
///
/// This register holds the user-defined threshold value for the interrupt generator.
/// The threshold is represented as a 16-bit signed integer.
/// The bit order can be configured via the `bit_order_msb` feature flag.
#[register(address = Reg::IntThsLReg, access_type = Iis2mdc, generics = 2)]
#[cfg_attr(feature = "bit_order_msb", bitfield(u16, order = Msb))]
#[cfg_attr(not(feature = "bit_order_msb"), bitfield(u16, order = Lsb))]
pub struct IntThsReg {
    /// Interrupt threshold value (16 bits, signed).
    #[bits(16)]
    pub int_ths: i16,
}

/// Operating modes for the sensor.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum Md {
    /// Continuous mode.
    #[default]
    ContinuousMode = 0,
    /// Single trigger mode.
    SingleTrigger = 1,
    /// Power down mode.
    PowerDown = 2,
}

/// Output data rates for the sensor.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum Odr {
    /// Output data rate of 10 Hz.
    #[default]
    _10hz = 0,
    /// Output data rate of 20 Hz.
    _20hz = 1,
    /// Output data rate of 50 Hz.
    _50hz = 2,
    /// Output data rate of 100 Hz.
    _100hz = 3,
}

/// Power modes for the sensor.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum Lp {
    /// High-resolution mode.
    #[default]
    HighResolution = 0,
    /// Low-power mode.
    LowPower = 1,
}

/// Low-pass filter bandwidth for the sensor.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum Lpf {
    /// Low-pass filter bandwidth of ODR/2
    #[default]
    OdrDiv2 = 0,
    /// Low-pass filter bandwidth of ODR/4
    OdrDiv4 = 1,
}

/// Reset pulse mode for the sensor.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum SetRst {
    /// Set/reset sensor every ODR/63.
    #[default]
    SetSensOdrDiv63 = 0,
    /// Set/reset sensor every ODR.
    SensOffCancEveryOdr = 1,
    /// Set/reset sensor only at power on.
    SetSensOnlyAtPowerOn = 2,
}

/// Data format options for the sensor (Big/Little Endian).
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum Ble {
    /// Least significant byte at lower address.
    #[default]
    LsbAtLowAdd = 0,
    /// Most significant byte at lower address.
    MsbAtLowAdd = 1,
}

/// Interrupt configuration options for data checks.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum IntOnDataOff {
    /// Check data before hard-iron correction.
    #[default]
    CheckBefore = 0,
    /// Check data after hard-iron correction.
    CheckAfter = 1,
}

/// I2C interface enable/disable options.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Default, TryFrom)]
#[try_from(repr)]
pub enum I2cDis {
    /// I2C interface enabled.
    #[default]
    Enable = 0,
    /// I2C interface disabled.
    Disable = 1,
}
