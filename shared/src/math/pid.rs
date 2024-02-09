use std::time::Instant;

pub struct PidController {
    k_p: f64,
    k_i: f64,
    k_d: f64,

    first_run: bool,
    previous_time: Instant,
    cumulative_error: f64,
    last_error: f64,
    last_output: f64,

    input_bounds: Option<(f64, f64)>,
    output_bounds: Option<(f64, f64)>,
    pub target_value: f64,
}

impl PidController {
    pub fn new(k_p: f64, k_i: f64, k_d: f64) -> Self {
        Self {
            k_p,
            k_i,
            k_d,
            first_run: true,
            previous_time: Instant::now(),
            cumulative_error: 0.0,
            last_error: 0.0,
            last_output: 0.0,
            input_bounds: None,
            output_bounds: None,
            target_value: 0.0,
        }
    }

    pub fn set_input_range(&mut self, min: f64, max: f64) {
        assert!(min < max);
        self.input_bounds = Some((min, max))
    }

    pub fn set_output_range(&mut self, min: f64, max: f64) {
        assert!(min < max);
        self.output_bounds = Some((min, max))
    }

    fn get_error(&self, input: f64) -> f64 {
        let mut error = self.target_value - input;

        if let Some((min_input, max_input)) = self.input_bounds {
            let input_range = max_input - min_input;

            while error.abs() > input_range / 2.0 {
                error -= error.signum() * input_range
            }
        }

        error
    }

    pub fn compute(&mut self, input: f64) -> f64 {
        let error = self.get_error(input);

        if self.first_run {
            self.first_run = false;

            self.last_error = error;
            self.previous_time = Instant::now();
            return 0.0;
        }

        let delta_time = self.previous_time.elapsed().as_millis() as f64;
        if delta_time == 0.0 {
            return 0.0;
        }

        {
            let (min, max) = self.output_bounds.unwrap_or((0.0, 0.0));

            if (min <= self.last_output && self.last_output <= max)
                || self.cumulative_error.signum() != error.signum()
            {
                self.cumulative_error += 0.5 * (error + self.last_error) * delta_time;
            }
        }

        let derivative = (error - self.last_error) / delta_time;

        let base_output =
            self.k_p * error + self.k_i * self.cumulative_error + self.k_d * derivative;
        let output = if base_output.abs() < 0.0001 {
            0.0
        } else {
            base_output
        };

        self.last_output = output;
        self.last_error = error;
        self.previous_time = Instant::now();

        if let Some((min, max)) = self.output_bounds {
            output.clamp(min, max)
        } else {
            output
        }
    }

    pub fn reset(&mut self) {
        self.previous_time = Instant::now();
        self.cumulative_error = 0.0;
        self.last_error = 0.0;
        self.last_output = 0.0;
    }
}
