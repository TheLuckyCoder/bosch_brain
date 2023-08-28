const TRIGGER: u8 = 24;
const ECHO: u8 = 23;

async fn main() -> anyhow::Result<()> {
    let mut ultrasonic = HcSr04::new(
        TRIGGER,
        ECHO,
        Some(20_f32) // Ambient temperature (if `None` defaults to 20.0C)
    )?;

    loop {
        // Perform distance measurement, specifying measuring unit of return value.
        match ultrasonic.measure_distance(Unit::Meters)? {
            Some(dist) => println!("Distance: {:.2}m", dist),
            None => println!("Object out of range"),
        }
    }
}