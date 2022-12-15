mod traffic_lights;

pub async fn run_listeners() {
    traffic_lights::run_listener(|traffic_lights| log::info!("Traffic Lights: {:?}", traffic_lights))
        .await
        .expect("TODO");
}
