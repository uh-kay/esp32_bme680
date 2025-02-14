use std::time::Duration;

use bme680::{Bme680, I2CAddress, IIRFilterSize, OversamplingSetting, PowerMode, SettingsBuilder};
use esp_idf_hal::{delay::Delay, i2c, peripherals::Peripherals};
use anyhow::Result;
use log::info;

fn main() -> Result<(), ()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let mut delay: Delay = Default::default();

    let peripherals = Peripherals::take().unwrap();
    
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let config = i2c::config::Config::new();
    
    let i2c = i2c::I2cDriver::new(peripherals.i2c0, sda, scl, &config).unwrap();

    let mut dev = Bme680::init(i2c, &mut delay, I2CAddress::Secondary).map_err(|e| {
        log::error!("Error at bme680 init {e:?}");
    })?;

    let settings = SettingsBuilder::new()
        .with_humidity_oversampling(OversamplingSetting::OS2x)
        .with_pressure_oversampling(OversamplingSetting::OS4x)
        .with_temperature_oversampling(OversamplingSetting::OS8x)
        .with_temperature_filter(IIRFilterSize::Size3)
        .with_gas_measurement(Duration::from_millis(1500), 320, 25)
        .with_temperature_offset(-2.2)
        .with_run_gas(true)
        .build();

    let profile_dur = dev.get_profile_dur(&settings.0).map_err(|e| {
        log::error!("Unable to get profile dur {e:?}");
    })?;
    info!("Profile duration {:?}", profile_dur);
    info!("Setting sensor settings");
    dev.set_sensor_settings(&mut delay, settings)
        .map_err(|e| {
            log::error!("Unable to apply sensor settings {e:?}");
        })?;
    info!("Setting forced power modes");
    dev.set_sensor_mode(&mut delay, PowerMode::ForcedMode)
        .map_err(|e| {
            log::error!("Unable to set sensor mode {e:?}");
        })?;

    let sensor_settings = dev.get_sensor_settings(settings.1);
    info!("Sensor settings: {:?}", sensor_settings);

    loop {
        delay.delay_ms(5000u32);
        let power_mode = dev.get_sensor_mode();
        info!("Sensor power mode: {:?}", power_mode);
        info!("Setting forced power modes");
        dev.set_sensor_mode(&mut delay, PowerMode::ForcedMode)
            .map_err(|e| {
                log::error!("Unable to set sensor mode {e:?}");
            })?;
        info!("Retrieving sensor data");
        let (data, _state) = dev.get_sensor_data(&mut delay).map_err(|e| {
            log::error!("Unable to get sensor data {e:?}");
        })?;
        info!("Sensor Data {:?}", data);
        info!("Temperature {}°C", data.temperature_celsius());
        info!("Pressure {}hPa", data.pressure_hpa());
        info!("Humidity {}%", data.humidity_percent());
        info!("Gas Resistence {}Ω", data.gas_resistance_ohm());
    }
}
