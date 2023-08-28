extern crate bno055;
use bno055::{BNO055, BNO055Config, CalibrationData, OperationMode, Vector};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the BNO055 sensor.
    println!("Starting BMO");
    let mut delay = Delay {};
    let i2c_1 = I2cdev::new("/dev/i2c-1").unwrap();

    let mut bno = bno055::Bno055::new(i2c_1).with_alternative_address();

    match bno.init(&mut delay) {
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

    let calibration_status = bno.get_calibration_status()?;
    println!("Calibration Status: {:?}", calibration_status);

    if calibration_status.system != 3 || calibration_status.gyro != 3 || calibration_status.accel != 3 {
        println!("Sensor calibration is needed.");

        // Set the sensor to CONFIG mode for calibration.
        bno.set_operation_mode(OperationMode::CONFIG)?;

        // Start calibration and wait until it's complete.
        start_calibration(&mut bno)?;
    }
    // Set the sensor to NDOF mode for normal operation.
    bno.set_operation_mode(OperationMode::NDOF)?;

    let mut previous_time = Instant::now();
    let mut velocity = Vector::default();

    loop {
        // Read acceleration data.
        let acceleration = bno.get_acceleration()?;

        // Calculate time elapsed since the last iteration.
        let current_time = Instant::now();
        let elapsed_time = current_time.duration_since(previous_time).as_secs_f32();
        previous_time = current_time;

        // Calculate velocity by integrating acceleration.
        velocity.x += acceleration.x * elapsed_time;
        velocity.y += acceleration.y * elapsed_time;
        velocity.z += acceleration.z * elapsed_time;

        // Display acceleration and velocity data.
        println!("Acceleration: {:?}", acceleration);
        println!("Velocity: {:?}", velocity);

        // Add a delay to control the data update rate.
        sleep(Duration::from_millis(100));
    }
}

fn start_calibration(bno: &mut BNO055) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let calibration_status = bno.get_calibration_status()?;
        println!("Calibration Status: {:?}", calibration_status);

        // Check if all three calibration values are 3 to indicate full calibration.
        if calibration_status.system == 3 && calibration_status.gyro == 3 && calibration_status.accel == 3 {
            println!("Sensor is fully calibrated.");
            break; // Exit the loop once fully calibrated.
        }

        sleep(Duration::from_secs(1)); // Wait for a second before checking again.
    }

    Ok(())
}
