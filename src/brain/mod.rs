use std::io::{BufRead, Read};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

use crate::brain::data::{update_data, BrainData};
use crate::math::pid::PidController;
use crate::serial;
use crate::serial::Message;

mod data;

pub fn start_brain() {
    let brain_data = Arc::new(Mutex::new(BrainData::default()));

    update_data(brain_data.clone());

    process_data(brain_data);
}

#[derive(Clone, Copy)]
enum LaneState {
    Follow,
    Stop,
}

enum ActionState {
    Intersection,
    LaneKeeping(LaneState),
    Parking,
}

impl ActionState {}

fn process_data(brain_data: Arc<Mutex<BrainData>>) {
    let mut last_data = BrainData::default();
    let mut lane_pid = PidController::new(0.8, 0.5, 0.01);
    let mut action_state = ActionState::LaneKeeping(LaneState::Follow);

    loop {
        thread::sleep(Duration::from_millis(1));

        let data = brain_data.lock().unwrap().clone();
        if last_data == data {
            continue;
        }
        last_data = data.clone();

        action_state = match action_state {
            ActionState::Intersection => {
                todo!("Intersection not implemented yet")
            }
            ActionState::LaneKeeping(lane) => lane_state(&data, &mut lane_pid, &lane),
            ActionState::Parking => {
                park_action();
                ActionState::LaneKeeping(LaneState::Follow)
            }
        }
    }
}

const STOP_SIGN_DISTANCE: f64 = 10.0;

fn lane_state(
    data: &BrainData,
    lane_pid: &mut PidController,
    lane_state: &LaneState,
) -> ActionState {
    if data.signs.stop < STOP_SIGN_DISTANCE {
        return ActionState::LaneKeeping(LaneState::Stop);
    }

    match lane_state {
        LaneState::Follow => {
            let total_angle = data.lanes_angle.left + data.lanes_angle.right;
            let steering_angle = lane_pid.compute(total_angle);
            serial::send(Message::Steer(steering_angle as f32));

            let speed = data.car.speed;
            let speed2 = if data.signs.crosswalk < 10.0 {
                speed * 0.7
            } else {
                speed
            };

            serial::send(Message::Speed(speed2 as f32));
        }
        LaneState::Stop => {
            serial::send(Message::Speed(0_f32));
            thread::sleep(Duration::from_secs(2));
            return ActionState::LaneKeeping(LaneState::Follow);
        }
    }

    ActionState::LaneKeeping(*lane_state)
}

fn park_action() {
    let file = std::fs::File::open("parking.psv").unwrap();

    io::BufReader::new(file).lines().for_each(|line| {
        let line = line.unwrap();
        let mut parts = line.split('|');
        let time = parts.next().unwrap().parse::<u64>().unwrap();
        let message = parts.next().unwrap();

        thread::sleep(Duration::from_millis(time));
        serial::send(Message::Raw(message.to_string()));
    });
}
