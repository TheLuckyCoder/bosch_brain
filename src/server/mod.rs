use tokio::task;

pub mod data;
mod moving_obstacle;
mod traffic_lights;

pub async fn run_server_listeners() -> std::io::Result<()> {
    let traffic = task::spawn(traffic_lights::run_listener(|traffic_lights| {
        log::info!("Traffic Lights: {:?}", traffic_lights)
    }));

    task::spawn(moving_obstacle::run_listener(|moving_obstacle| {
        log::info!("MovingObstacle: {}", moving_obstacle)
    }))
    .await??;

    traffic.await??;

    Ok(())
}
