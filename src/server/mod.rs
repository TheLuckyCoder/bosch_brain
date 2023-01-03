use tokio::task;

pub mod data;
mod environment;
mod localisation;
mod moving_obstacle;
mod traffic_lights;
mod utils;

pub async fn run_server_listeners() -> std::io::Result<()> {
    let localization = task::spawn(localisation::run_localization(|robot_pos| {
        log::info!("Robot Pos: {:?}", robot_pos)
    }));

    let traffic = task::spawn(traffic_lights::run_listener(|traffic_lights| {
        log::info!("Traffic Lights: {:?}", traffic_lights)
    }));

    task::spawn(moving_obstacle::run_listener(|moving_obstacle| {
        log::info!("MovingObstacle: {}", moving_obstacle)
    }))
    .await??;

    traffic.await??;
    localization.await??;

    Ok(())
}
