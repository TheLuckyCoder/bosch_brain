// use std::thread::sleep;
// use std::time::Duration;
// use hc_sr04::{HcSr04, Unit};
//
// const TRIGGER: u8 = 24;
// const ECHO: u8 = 23;
//
// fn main() {
//     // Initialize the sensor
//     let mut sensor = HcSr04::new(TRIGGER, ECHO, Some(20_f32)).expect("Failed to initialize HC-SR04");
//
//
//     loop {
//         // Perform a distance measurement
//         if let Ok(distance) = sensor.measure_distance(Unit::Centimeters) {
//             println!("Distance: {:?} cm", distance);
//         } else {
//             println!("Measurement error");
//         }
//
//         // Wait for a short duration before taking the next measurement
//         sleep(Duration::from_secs(1));
//     }
// }


use std::io::{BufRead, stdin};
use std::thread;
use std::time::Duration;
use hc_sr04::{HcSr04, Unit};
use linux_embedded_hal::{Delay, I2cdev};
use pwm_pca9685::{Address, Channel, Pca9685};
use std::time::{SystemTime, UNIX_EPOCH};

const TRIGGER: u8 = 24;
const ECHO: u8 = 23;

fn main() -> anyhow::Result<()> {
    println!("Starting PWM");
    let i2c_1 = I2cdev::new("/dev/i2c-1").unwrap();
    let address = Address::default();
    let mut pwm = Pca9685::new(i2c_1, address).unwrap();

    // This corresponds to a frequency of 60 Hz.
    pwm.set_prescale(100).unwrap();

    // It is necessary to enable the device.
    pwm.enable().unwrap();

    println!("PWM Enabled");

    println!("Starting BMO");
    let mut delay = Delay {};
    let i2c_1 = I2cdev::new("/dev/i2c-1").unwrap();

    let mut imu = bno055::Bno055::new(i2c_1).with_alternative_address();

    match imu.init(&mut delay) {
        Ok(_) => {
            // Initialization successful, continue with your code
            match imu.set_mode(bno055::BNO055OperationMode::NDOF, &mut delay) {
                Ok(_) => {
                    // Mode set successfully, continue with your code
                    println!("BNO Enabled");
                },
                Err(err) => {
                    eprintln!("Error setting mode: {:?}", err);
                    // Handle the error appropriately
                }
            }
        },
        Err(err) => {
            eprintln!("Error initializing IMU: {:?}", err);
            // Handle the error appropriately
        }
    }

    // println!("Starting Ultrasonic on pins trigger: {TRIGGER} echo: {ECHO}");
    // let mut ultrasonic = HcSr04::new(
    //     TRIGGER,
    //     ECHO,
    //     Some(20_f32) // Ambient temperature (if `None` defaults to 20.0C)
    // )?;

    let time_step = 100;

    loop {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

        let seconds = duration.as_secs();
        let nanos = duration.subsec_nanos();
        println!("Timestamp: {} seconds {} nanoseconds", seconds, nanos);

        // Read sensor data in the loop
        match imu.quaternion() {
            Ok(quat) => {
                println!("Quaternion: {:?}", quat);
            },
            Err(err) => {
                eprintln!("Error reading Quaternion: {:?}", err);
                // Handle the error appropriately
            }
        }

        println!("Duty Cycle %: ");
        let x: i32 = stdin().lock().lines().next().unwrap()?.parse()?;
        // let x = 0;
        for index in 5000 / 16..7000 / 16 {
            pwm.set_channel_on_off(Channel::C15, 0, index).unwrap();
            pwm.set_channel_on_off(Channel::C4, 0, (x as f32 * 40.96f32) as u16).unwrap();
            thread::sleep(Duration::from_millis(20));
        }

        for index in (7000 / 16..5000 / 16).rev() {
            pwm.set_channel_on_off(Channel::C15, 0, index).unwrap();
            pwm.set_channel_on_off(Channel::C4, 0, (x as f32 * 40.96f32) as u16).unwrap();
            thread::sleep(Duration::from_millis(20));
        }

        // Perform distance measurement, specifying measuring unit of return value.
        // match ultrasonic.measure_distance(Unit::Meters)? {
        //     Some(dist) => println!("Distance: {:.2}m", dist),
        //     None => println!("Object out of range"),
        // }

    }

    let _dev = pwm.destroy(); // Get the I2C device back

    Ok(())
}

// #![allow(dead_code)]
//
// use env_logger::Env;
// use std::fs::OpenOptions;
// use std::io::BufRead;
// use std::path::Path;
// use std::thread::sleep;
// use tokio::task;
//
// use crate::serial::Message;
//
// mod brain;
// mod math;
// mod serial;
// mod server;
// #[cfg(test)]
// mod tests;
// mod track;
//
// struct Cleanup;
//
// impl Drop for Cleanup {
//     fn drop(&mut self) {
//         serial::send_blocking(Message::Speed(0_f32)).expect("Failed to force stop car");
//         println!("Car stopped");
//     }
// }
//
// #[tokio::main]
// async fn main() -> std::io::Result<()> {
//     // let _cleanup = Cleanup;
//
//     env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
//         .format_timestamp(None)
//         .target(env_logger::Target::Stdout)
//         .init();
//
//     // let track = track::get_track();
//
//     let path = "/home/car/recorded_movements/full_run.txt";
//
//     task::spawn(async move {
//         if let Err(e) = server::steering_wheel::run_steering_wheel_server(path).await {
//             log::error!("Steering wheel server error: {e}");
//         }
//     });
//     // if Path::new(path).exists() {
//     //     let file = OpenOptions::new().read(true).open(path)?;
//     //
//     //     // read all lines from file and store them in a Vec
//     //     let lines: Vec<_> = std::io::BufReader::new(file)
//     //         .lines()
//     //         .map(|l| l.unwrap())
//     //         .collect();
//     //
//     //     for line in lines {
//     //         let mut split = line.split('|');
//     //         let time = split.next().unwrap();
//     //         let message = split.next().unwrap();
//     //
//     //         serial::send_blocking(Message::Raw(message.to_string()))?;
//     //         //sleep for time milliseconds
//     //         sleep(std::time::Duration::from_millis(
//     //             time.parse::<u64>().unwrap(),
//     //         ));
//     //     }
//     // }
//     // serial::send_blocking(Message::Speed(0_f32))?; //stop car
//     brain::start_brain();
//
//     Ok(())
// }
