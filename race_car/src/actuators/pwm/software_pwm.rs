use crate::actuators::pwm::Percentage;
use std::io;

use std::net::TcpStream;
use tracing::info;

const PWM_MAX_RANGE: u32 = 40000; // duty cycle will be in the range of 0-40000

pub struct PiGpioDmaPwm {
    period: f64,
    gpio_pin: u8,
    max_duty_cycle: u32,
    tcp_stream: TcpStream,
}

impl PiGpioDmaPwm {
    /// Creates a new PiGpioDmaPwm instance.
    /// frequency is expected to be in the range of 50Hz to 50kHz.
    /// Hobby servos typically use 50Hz, while motor controllers can use up to 50kHz.
    pub fn new(gpio_pin: u8, pi_gpio_daemon_port: u16, frequency: u32) -> io::Result<Self> {
        let frequency = frequency.clamp(50, 50000);

        let mut tcp_stream = TcpStream::connect(("127.0.0.1", pi_gpio_daemon_port))?;
        pi_gpio::set_mode(&mut tcp_stream, gpio_pin, pi_gpio::Mode::Output)?;
        let max_duty_cycle = pi_gpio::set_pwm_range(&mut tcp_stream, gpio_pin, PWM_MAX_RANGE)?;
        info!("Set duty cycle range to {}", max_duty_cycle);
        let frequency = pi_gpio::set_pwm_frequency(&mut tcp_stream, gpio_pin, frequency)?;
        info!("Set frequency to {}", frequency);

        let result = Self {
            period: 1.0 / frequency as f64,
            gpio_pin,
            max_duty_cycle,
            tcp_stream,
        };

        Ok(result)
    }

    pub fn set_duty_cycle(&mut self, percentage: Percentage) -> io::Result<()> {
        let duty_cycle = ((percentage.value / 100.0) * (PWM_MAX_RANGE as f64)) as u32;
        info!("duty cycle set to {}", duty_cycle);
        pi_gpio::set_pwm_duty_cycle(
            &mut self.tcp_stream,
            self.gpio_pin,
            duty_cycle.min(PWM_MAX_RANGE),
        ).map(|_| ())
    }
}

mod pi_gpio {
    use std::io;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use tracing::info;

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
        SetDutyCycle = 5,
        SetPwmRange = 6,
        SetFrequency = 7,
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
        info!("Sent packet");

        // Read 4-byte response
        let mut response = [0u8; 16];
        stream.read_exact(&mut response)?;
        info!("Got {:?}", &response);

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
