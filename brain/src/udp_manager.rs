use sensors::SensorManager;
use std::net::UdpSocket;

struct UdpManager {
    currently_active_sensor: Option<String>,
}

impl UdpManager {
    pub fn new() -> std::io::Result<Self> {
        let server = UdpSocket::bind("0.0.0.0:3000")?;
        server.connect("0.0.0.0:3001")?;

        // std::thread::spawn(|| {
        //     server.send().unwrap();
        // });

        Ok(Self {
            currently_active_sensor: None,
        })
    }
}
