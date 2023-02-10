use crate::brain::data::{update_data, BrainData};
use crate::math::pid::PidController;
use crate::serial;
use crate::serial::Message;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod data;

pub fn start_brain() {
    let brain_data = Arc::new(Mutex::new(BrainData::default()));

    update_data(brain_data.clone());

    process_data(brain_data);
}

fn process_data(brain_data: Arc<Mutex<BrainData>>) {
    let mut last_data = BrainData::default();
    let mut lane_pid = PidController::new(1.0, 0.5, 0.01);

    loop {
        std::thread::sleep(Duration::from_millis(1));

        let data = brain_data.lock().unwrap().clone();
        if last_data == data {
            continue;
        }
        last_data = data.clone();

        let total_angle = data.lanes_angle.left + data.lanes_angle.right;
        let steering_angle = lane_pid.compute(total_angle);
        serial::send(Message::Steer(steering_angle as f32));
    }
}
