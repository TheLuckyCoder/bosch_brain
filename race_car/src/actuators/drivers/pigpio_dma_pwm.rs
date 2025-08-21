use crate::actuators::drivers::{DutyCycle, PwmDriver};
use std::io;

use std::net::TcpStream;
use tracing::info;

const PWM_MAX_RANGE: u32 = 100;

pub struct PiGpioDmaPwm {
    gpio_pin: u8,
    max_duty_cycle: u32,
    tcp_stream: TcpStream,
}

impl PiGpioDmaPwm {
    /// Creates a new `PiGpioDmaPwm` instance.
    ///
    /// # Parameters
    /// - `gpio_pin`: The GPIO pin to output PWM on.
    /// - `pi_gpio_daemon_port`: The port where the PiGPIO daemon is listening (usually 8888).
    /// - `frequency`: The desired PWM frequency in Hz. Valid range is 50–50,000 Hz.
    ///
    /// # Returns
    /// Returns an `io::Result` containing the initialized `PiGpioDmaPwm`.
    ///
    /// # Notes
    /// - Hobby servos typically use 50 Hz.
    /// - Motor controllers can use frequencies up to 50 kHz.
    /// - The actual PWM frequency returned by the daemon may differ slightly from the requested value.
    pub fn new(gpio_pin: u8, pi_gpio_daemon_port: u16, frequency: u32) -> io::Result<Self> {
        // Clamp frequency to safe range
        let frequency = frequency.clamp(50, 50_000);

        // Connect to the PiGPIO daemon
        let mut tcp_stream = TcpStream::connect(("127.0.0.1", pi_gpio_daemon_port))?;

        // Set GPIO pin mode to output
        pi_gpio::set_mode(&mut tcp_stream, gpio_pin, pi_gpio::Mode::Output)?;

        // Set PWM range and get the actual max duty cycle
        let max_duty_cycle = pi_gpio::set_pwm_range(&mut tcp_stream, gpio_pin, PWM_MAX_RANGE)?;
        info!("Set duty cycle range for GPIO {} to {}", gpio_pin, max_duty_cycle);


        // Set PWM frequency and get the actual frequency applied
        let actual_frequency = pi_gpio::set_pwm_frequency(&mut tcp_stream, gpio_pin, frequency)?;
        info!("Set PWM frequency for GPIO {} to {} Hz", gpio_pin, actual_frequency);


        pi_gpio::set_pwm_duty_cycle(&mut tcp_stream, gpio_pin, 0)?;

        Ok(Self {
            gpio_pin,
            max_duty_cycle,
            tcp_stream,
        })
    }
}

impl PwmDriver for PiGpioDmaPwm {
    fn set_duty_cycle(&mut self, duty_cycle: DutyCycle) {
        let fraction = duty_cycle.as_fraction(); // 0.0–1.0
        let duty = (fraction * self.max_duty_cycle as f64) as u32;

        info!("Set PWM duty cycle for GPIO {} to {} (fraction: {})", self.gpio_pin, duty, fraction);

        if let Err(e) = pi_gpio::set_pwm_duty_cycle(
            &mut self.tcp_stream,
            self.gpio_pin,
            duty.min(self.max_duty_cycle),
        ) {
            eprintln!("Failed to set PWM duty cycle for GPIO {}: {}", self.gpio_pin, e);
            // optionally: return early or handle differently
        }
    }

    fn turn_off(&mut self) {
        if let Err(e) = pi_gpio::set_pwm_duty_cycle(&mut self.tcp_stream, self.gpio_pin, 0) {
            eprintln!("Failed to turn off PWM for GPIO {}: {}", self.gpio_pin, e);
        }
    }
}


mod pi_gpio {
    use std::io;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    #[repr(u32)]
    pub enum Mode {
        Input = 0,
        Output = 1,
    }

    pub fn set_mode(stream: &mut TcpStream, gpio_pin: u8, mode: Mode) -> io::Result<()> {
        send_pigpio_command(stream, PwmCommand::SetMode, gpio_pin as u32, mode as u32).map(|_| ())
    }

    /// returns current duty cycle if successful
    pub fn set_pwm_duty_cycle(
        stream: &mut TcpStream,
        gpio_pin: u8,
        duty_cycle: u32,
    ) -> io::Result<u32> {
        send_pigpio_command(
            stream,
            PwmCommand::SetDutyCycle,
            gpio_pin as u32,
            duty_cycle,
        )
    }

    /// Returns the actual range for the current gpio frequency
    pub fn set_pwm_range(stream: &mut TcpStream, gpio_pin: u8, range: u32) -> io::Result<u32> {
        send_pigpio_command(stream, PwmCommand::SetPwmRange, gpio_pin as u32, range)
    }

    /// Returns the numerically closest frequency
    pub fn set_pwm_frequency(
        stream: &mut TcpStream,
        gpio_pin: u8,
        frequency: u32,
    ) -> io::Result<u32> {
        send_pigpio_command(
            stream,
            PwmCommand::SetFrequency,
            gpio_pin as u32,
            frequency,
        )
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq)]
    enum PwmCommand {
        SetMode = 0,
        Write = 4,
        SetDutyCycle = 5,
        SetPwmRange = 6,
        SetFrequency = 7,
    }

    pub fn write(stream: &mut TcpStream, gpio_pin: u8, level: bool) -> io::Result<()> {
        send_pigpio_command(
            stream,
            PwmCommand::Write,
            gpio_pin as u32,
            if level { 1 } else { 0 },
        ).map(|_| ())
    }

    fn send_pigpio_command(
        stream: &mut TcpStream,
        command: PwmCommand,
        param_1: u32,
        param_2: u32,
    ) -> io::Result<u32> {
        let mut packet = Vec::with_capacity(16);
        packet.extend((command as u32).to_le_bytes());
        packet.extend(param_1.to_le_bytes());
        packet.extend(param_2.to_le_bytes());
        packet.extend(0_u32.to_le_bytes());

        // Send command
        stream.write_all(&packet)?;

        // Read 4-byte response
        let mut response = [0u8; 16];
        stream.read_exact(&mut response)?;

        let result = i32::from_le_bytes([response[12], response[13], response[14], response[15]]);
        if result < 0 {
            return Err(io::Error::other(
                format!("Failed run PWM command {command:?} with error code: {result},\
                 check https://github.com/joan2937/pigpio/blob/c33738a320a3e28824af7807edafda440952c05d/pigpio.py#L579"),
            ));
        }

        Ok(result as u32)
    }
}
