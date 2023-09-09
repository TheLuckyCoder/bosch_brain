enum State {
    Standby,
    RemoteControl,
    AutonomousControl,
}

pub struct StateMachine {
    state: State,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: State::Standby,
        }
    }
    
    pub fn to_standby(&mut self) {
        self.state = State::Standby;
    }

    pub fn to_remote_controlled(&mut self) {
        self.state = State::RemoteControl;
    }

    pub fn to_autonomous_controlled(&mut self) {
        self.state = State::AutonomousControl;
    }
}
