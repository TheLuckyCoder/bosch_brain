use std::time::{Duration, SystemTime};

use multiqueue2::BroadcastReceiver;

use shared::math::AlmostEquals;

use crate::sensors::TimedSensorData;
use sensors::name::SensorName;
use sensors::{HardwareSensor, SensorData};

/// Tunables for converting Motor RPS to linear velocity and fusing with IMU-derived velocity.
#[derive(Debug, Clone, Copy)]
pub struct VelocityParams {
    /// Motor shaft rotations per one wheel rotation.
    pub gear_ratio: f64,
    /// Wheel diameter in meters.
    pub wheel_diameter_m: f64,
    /// If |v_rps| < this, treat as stopped for IMU gating (m/s).
    pub stop_threshold_mps: f64,
    /// Complementary filter weight for the tachometer (0..=1). 1 → all MotorRPS, 0 → all IMU.
    pub alpha_tacho: f64,
    /// Dead-zone for IMU acceleration in m/s^2 to suppress micro-noise.
    pub imu_acc_deadzone: f64,
    /// Optional slip factor to down-scale tachometer velocity if you see systemic over-read.
    pub slip_scale: f64,
}

impl Default for VelocityParams {
    fn default() -> Self {
        Self {
            gear_ratio: 1.0,
            wheel_diameter_m: 0.21 / std::f64::consts::PI, // if you previously had circumference=0.21
            stop_threshold_mps: 0.02,
            alpha_tacho: 0.8,
            imu_acc_deadzone: 0.06, // roughly 0.006 g
            slip_scale: 1.0,
        }
    }
}

pub struct VirtualVelocitySensor {
    receiver: BroadcastReceiver<TimedSensorData>,
    params: VelocityParams,
    // IMU integration state
    last_velocity_imu: f64,
    last_accel_x: f64,
    last_time: Option<SystemTime>,
    // Last observed MotorRPS
    last_motor_rps: Option<(f64, SystemTime)>,
}

impl VirtualVelocitySensor {
    pub fn new(receiver: BroadcastReceiver<TimedSensorData>, params: VelocityParams) -> Self {
        Self {
            receiver,
            params,
            last_velocity_imu: 0.0,
            last_accel_x: 0.0,
            last_time: None,
            last_motor_rps: None,
        }
    }

    #[inline]
    fn wheel_circumference(&self) -> f64 {
        std::f64::consts::PI * self.params.wheel_diameter_m
    }

    /// Update IMU-integrated velocity using trapezoidal rule, optionally gated by tacho stop.
    fn update_imu_velocity(&mut self, accel_x_mps2: f64, ts: SystemTime, v_tacho_mps_opt: Option<f64>) {
        let dt = if let Some(t_prev) = self.last_time {
            ts.duration_since(t_prev).unwrap_or_else(|_| Duration::from_secs(0)).as_secs_f64()
        } else {
            0.0
        };
        self.last_time = Some(ts);

        // Apply dead-zone to acceleration
        let ax = if accel_x_mps2.almost_equals(0.0, self.params.imu_acc_deadzone) {
            0.0
        } else {
            accel_x_mps2
        };

        // If tachometer says we are basically stopped, strongly damp IMU integration drift.
        if let Some(v_tacho) = v_tacho_mps_opt {
            if v_tacho.abs() < self.params.stop_threshold_mps {
                // Snap to zero and reset integrator to fight drift when stationary.
                self.last_velocity_imu = 0.0;
                self.last_accel_x = 0.0;
                return;
            }
        }

        if dt > 0.0 {
            // Trapezoidal integration of acceleration
            let v_new = self.last_velocity_imu + 0.5 * (ax + self.last_accel_x) * dt;
            self.last_velocity_imu = v_new;
        }
        self.last_accel_x = ax;
    }

    fn velocity_from_motor_rps(&self, motor_rps: f64) -> f64 {
        let wheel_rps = motor_rps / self.params.gear_ratio;
        let v = wheel_rps * self.wheel_circumference();
        v * self.params.slip_scale
    }
}

impl HardwareSensor for VirtualVelocitySensor {
    fn name(&self) -> SensorName { SensorName::VirtualVelocity }

    fn prepare_read(&mut self) {
        // Reset integrator each measurement cycle if your architecture expects stateless reads.
        // Remove this if you want continuous integration across reads.
        self.last_velocity_imu = 0.0;
        self.last_accel_x = 0.0;
        self.last_time = None;
    }

    fn read_data(&mut self) -> SensorData {
        let mut latest_accel: Option<(f64, SystemTime)> = None;
        let mut latest_motor_rps: Option<(f64, SystemTime)> = None;

        // Pull all queued sensor updates and keep the most recent of each type
        for sd in self.receiver.try_iter() {
            match sd.data {
                SensorData::Imu { acceleration, .. } => {
                    // Use X-axis; adjust if your forward axis is different
                    let ax = acceleration[0] as f64;
                    latest_accel = Some((ax, sd.timestamp));
                }
                SensorData::MotorRPS(rps) => {
                    latest_motor_rps = Some((rps, sd.timestamp));
                    self.last_motor_rps = latest_motor_rps;
                }
                _ => {}
            }
        }

        // Compute tachometer velocity if we have a reading (prefer newest)
        let v_tacho_mps = latest_motor_rps
            .or(self.last_motor_rps) // fallback to last known if nothing new arrived this tick
            .map(|(rps, _)| self.velocity_from_motor_rps(rps));

        // Update IMU integrator with newest accel (gated by tacho stop)
        if let Some((ax, ts)) = latest_accel {
            self.update_imu_velocity(ax, ts, v_tacho_mps);
        }

        let v_imu = self.last_velocity_imu;
        let v_rps = v_tacho_mps.unwrap_or(0.0);

        // Complementary fusion: tachometer (low-drift) vs IMU (high-bandwidth)
        let alpha = self.params.alpha_tacho.clamp(0.0, 1.0);
        let v_fused = alpha * v_rps + (1.0 - alpha) * v_imu;

        // Return all three for downstream consumers
        SensorData::VirtualVelocity { imu: v_imu, rot_encoder: v_rps, fusion: v_fused }
    }
}