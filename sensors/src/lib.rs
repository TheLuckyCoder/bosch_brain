use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use hc_sr04::{HcSr04, Unit};

use imu::{ImuSpecs, Vec3};

pub mod imu;

pub enum SensorData {
    Distance(Option<f32>),
    Acceleration(Vec3),
    Gyroscope(Vec3),
}

fn get_distance_sensors(
    ambient_temperature: f32,
) -> Result<(HcSr04, HcSr04), hc_sr04::error::Error> {
    // Initialize driver.
    let sensor_1 = HcSr04::new(24, 23, Some(ambient_temperature))?;
    let sensor_2 = HcSr04::new(26, 25, Some(ambient_temperature))?;

    Ok((sensor_1, sensor_2))
}

pub fn get_sensor_data() -> Result<Receiver<SensorData>, String> {
    let (tx, rx) = channel();
    let mut imu = imu::get_imu()?;
    let (mut sensor_1, mut sensor_2) = get_distance_sensors(23_f32).map_err(|e| e.to_string())?;

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(100));
        // TODO perhaps add error handling to the sender
        let distance_1 = sensor_1
            .measure_distance(Unit::Centimeters)
            .map_err(|e| format!("Failed to read distance sensor 1: {e}"))
            .unwrap();
        let distance_2 = sensor_2
            .measure_distance(Unit::Centimeters)
            .map_err(|e| format!("Failed to read distance sensor 2: {e}"))
            .unwrap();

        // TODO process the values?

        tx.send(SensorData::Distance(distance_1)).unwrap();
        tx.send(SensorData::Acceleration(imu.get_acceleration()))
            .unwrap();
        tx.send(SensorData::Gyroscope(imu.get_gyroscope())).unwrap();
    });

    Ok(rx)
}
