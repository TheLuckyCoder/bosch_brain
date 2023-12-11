pub struct Gps {
    buffer: Vec<u8>,
}

impl Gps {
    pub fn new() -> anyhow::Result<>{
        let mut serial = serialport::new(
            "/dev/serial/by-id/usb-SEGGER_J-Link_000760170010-if00",
            115200,
        )
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_millis(200))
            .open_native()?;

        info!("Serial initialized");

        while let Err(e) = serial.write_all(b"\n\n") {
            error!("{e}");
            sleep(Duration::from_secs(1));
        }
        info!("Wrote!");
        while let Err(e) = serial.write_all(b"les\n") {
            error!("{e}");
            sleep(Duration::from_secs(1));
        }

        loop {
            match serial.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    std::io::stdout().write_all(&serial_buf[..t]).unwrap()
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    if let Err(err) = serial.write_all(b"\n") {
                        error!("Write error: {err}");
                    }
                },
                Err(e) => error!("Read error: {:?}", e),
            }
        }
    }

    pub fn read(&mut self) {
        let mut serial_buf: Vec<u8> = vec![0; 1000];

        //

        let mut line = String::new();

        loop {
            sleep(Duration::from_millis(200)).await;
            if serial.bytes_to_read()? == 0 {
                info!("Skipping");
                continue;
            }
            match serial.read_to_string(&mut line) {
                Ok(size) => info!("Read {size} chars: {line}"),
                Err(e) => {
                    error!("Read error: {e}");
                    if let Err(err) = serial.write_all(b"\n") {
                        error!("{err}")
                    }
                }
            };
        }
    }
}

impl BasicSensor for Gps {

}
