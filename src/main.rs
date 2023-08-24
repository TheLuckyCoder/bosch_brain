use std::io;
use std::sync::{Arc, Mutex};
use hc_sr04::{HcSr04, Unit};
use linux_embedded_hal::{Delay, I2cdev};
use pwm_pca9685::{Address, Channel, Pca9685};
use std::time::{SystemTime, UNIX_EPOCH};

struct MotorSettings {
    bonnet_channel: Channel,
    percentage_minimum: f64,
    percentage_middle: f64,
    percentage_maximum: f64,
}

const TRIGGER: u8 = 24;
const ECHO: u8 = 23;

///Will need to set these to correct values
const STEPPER_MOTOR: MotorSettings = MotorSettings {
    bonnet_channel: Channel::C15,
    percentage_minimum: 5.0,
    percentage_middle: 8.0,
    percentage_maximum: 13.0,
};
const DC_MOTOR: MotorSettings = MotorSettings {
    bonnet_channel: Channel::C4,
    percentage_minimum: 5.0,
    percentage_middle: 8.0,
    percentage_maximum: 13.0,
};

fn map_from_percentage_to_12_bit_int(input: f64) -> u64 {
    // Maps an input floating-point number that is between 0.0-100.0 (percentage) to 0-4096

    // Ensure input is within the 0.0-100.0 range
    let clamped_input = input.clamp(0.0, 100.0);

    // Map clamped_input to the 0-4096 range
    ((clamped_input * 40.96) as u64)
}

pub fn set_motor_input(pwm: &mut Pca9685<I2cdev>, input: f64, motor_settings: MotorSettings) {
    //Maps an input number that is between -1 and 1 (float) to a percentage than can't be smaller than percentage_minimum and bigger than percentage_maximum
    //If the input is smaller than -1 or bigger than 1 it gives equivalent to it (percentage_minimum/maximum)
    let clamped_input = input.clamp(-1.0, 1.0);

    let motor_input_percentage:f64 = match clamped_input {
        0.0 => {
            motor_settings.percentage_middle
        }
        -1.0..=0.0 => {
            (clamped_input * (motor_settings.percentage_middle - motor_settings.percentage_minimum)
                + motor_settings.percentage_minimum)
        }
        0.0..=1.0 => {
            (clamped_input * (motor_settings.percentage_maximum - motor_settings.percentage_middle)
                + motor_settings.percentage_middle)
        }
        _ => {
            motor_settings.percentage_middle
        }
    };

    pwm.set_channel_on_off(motor_settings.bonnet_channel, 0, map_from_percentage_to_12_bit_int(motor_input_percentage) as u16).unwrap();
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    println!("Starting Ultrasonic on pins trigger: {TRIGGER} echo: {ECHO}");
    let mut ultrasonic = HcSr04::new(
        TRIGGER,
        ECHO,
        Some(20_f32) // Ambient temperature (if `None` defaults to 20.0C)
    )?;

    std::panic::set_hook(Box::new(|panic_info| {
        println!("Panic occurred: {:?}", panic_info);

        pwm.set_channel_on_off(Channel::C4, 0, map_from_percentage_to_12_bit_int(DC_MOTOR.percentage_middle) as u16).unwrap();
        pwm.set_channel_on_off(Channel::C15, 0, map_from_percentage_to_12_bit_int(STEPPER_MOTOR.percentage_middle) as u16).unwrap();

        let _dev = pwm.destroy(); // Get the I2C device back
    }));

    let mut steering_percentage = 0f64;

    // Create an Arc and Mutex to share the user input between threads
    let user_input = Arc::new(Mutex::new(None));


    loop {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

        let seconds = duration.as_secs();
        let nanos = duration.subsec_nanos();
        println!("Timestamp: {} seconds {} nanoseconds", seconds, nanos);

        let input_thread = {
            let user_input = Arc::clone(&user_input);
            std::thread::spawn(move || {
                loop {
                    let mut input = String::new();
                    println!("Enter a number between 0 and 100:");
                    io::stdin().read_line(&mut input).expect("Failed to read line");

                    // Parse the input as a f64
                    let parsed_input: Result<f64, _> = input.trim().parse();

                    match parsed_input {
                        Ok(num) if num >= 0.0 && num <= 100.0 => {
                            // Input is valid, store it in the shared data
                            let mut user_input = user_input.lock().unwrap();
                            *user_input = Some(num);
                            break; // Exit the loop
                        }
                        Ok(_) => println!("Input is not between 0.0 and 100.0 please try again."),
                        Err(_) => println!("Invalid input. Please enter a number."),
                    }
                }
            })
        };

        // Wait for the input thread to finish
        input_thread.join().expect("Input thread panicked");

        // Access the user input from the main thread
        let locked_user_input = user_input.lock().unwrap();
        steering_percentage = *locked_user_input.as_ref().expect("No valid input provided.");

        pwm.set_channel_on_off(Channel::C15, 0, map_from_percentage_to_12_bit_int(steering_percentage) as u16).unwrap();
        // pwm.set_channel_on_off(Channel::C4, 0, map_from_percentage_to_12_bit_int(speed_percentage) as u16).unwrap();

        // 0 ... 5000  ... 7000   ... 65536
        // 0 ... 312.5 ... 437.5  ... 4096
        // 0 ... 7.62% ... 10.68% ... 100
        // pwm.set_channel_on_off(Channel::C4, 0, (x as f32 * 40.96f32) as u16).unwrap();

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

        // Perform distance measurement, specifying measuring unit of return value.
        match ultrasonic.measure_distance(Unit::Meters)? {
            Some(dist) => println!("Distance: {:.2}m", dist),
            None => println!("Object out of range"),
        }

    }
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
