use tokio::task;

pub mod data;
mod traffic_lights;
mod vehicle_to_vehicle;

pub async fn run_server_listeners() {
    let f = task::spawn(async move {
        traffic_lights::run_listener(|traffic_lights| {
            log::info!("Traffic Lights: {:?}", traffic_lights)
        })
        .await
        .expect("")
    });

    task::spawn(async move {
        vehicle_to_vehicle::run_listener(|vehicle_to_vehicle| {
            log::info!("Vehicle to vehicle: {}", vehicle_to_vehicle)
        })
        .await
        .expect("TODO");
    })
    .await
    .unwrap();

    f.await.unwrap();
}
