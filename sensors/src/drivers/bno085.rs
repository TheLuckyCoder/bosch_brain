// src/bno085.rs

use std::io::Read;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits, TTYPort};
use tracing::{error, info, warn};

use crate::name::SensorName;
use crate::{HardwareSensor, SensorData};

/// One BNO08x UART-RVC frame after 0xAA 0xAA header.
#[derive(Debug, Clone, Copy)]
struct RvcPacket {
    index: u8,
    /// degrees (ZYX: yaw around +Z, pitch +Y, roll +X)
    yaw_deg: f32,
    pitch_deg: f32,
    roll_deg: f32,
    /// m/s^2 (sensor/body frame)
    accel_ms2: [f32; 3],
    checksum_ok: bool,
}

/// Axis mapping: permute and sign-flip a 3D vector.
/// We use this to align the gravity computed from YPR to the accelerometer axes.
#[derive(Clone, Copy, Debug)]
struct AxisMap {
    perm: [usize; 3],
    sign: [f32; 3],
}

impl AxisMap {
    #[inline]
    fn apply(&self, v: [f32; 3]) -> [f32; 3] {
        let p = self.perm;
        let s = self.sign;
        [v[p[0]] * s[0], v[p[1]] * s[1], v[p[2]] * s[2]]
    }
}

/// BNO08x in UART-RVC mode (115200-8-N-1, ~100 Hz).
pub struct Bno085RvcSensor {
    port: Box<dyn SerialPort + Send>,
    last_index: Option<u8>,
    axis_map: AxisMap, // maps gravity (from YPR) to accelerometer axes
}

impl Bno085RvcSensor {
    /// `port_path` example on Pi: "/dev/serial0" or "/dev/ttyAMA0"
    pub fn new(port_path: &str) -> Result<Self> {
        // Use open_native() to get a TTYPort (Send), then box it as dyn SerialPort + Send.
        let tty: TTYPort = serialport::new(port_path, 115_200)
            .timeout(Duration::from_millis(50))
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
            .open_native()
            .with_context(|| format!("Failed to open serial port {port_path}"))?;

        let mut this = Self {
            port: Box::new(tty),
            last_index: None,
            axis_map: AxisMap {
                perm: [0, 1, 2],
                sign: [1.0, 1.0, 1.0],
            },
        };

        // Clear any junk bytes
        let _ = this.port.clear(serialport::ClearBuffer::All);

        // Auto-detect axis mapping over ~1.5 seconds while stationary.
        this.calibrate_axis_map(150, Duration::from_secs_f32(2.0));

        Ok(this)
    }

    /// Read one RVC frame; blocks until a full valid frame or timeout error.
    fn read_frame(&mut self) -> Result<RvcPacket> {
        // Seek 0xAA 0xAA header (allow arbitrary junk before it)
        let mut byte = [0u8; 1];
        let mut prev = 0u8;

        loop {
            self.port.read_exact(&mut byte)?;
            if prev == 0xAA && byte[0] == 0xAA {
                break;
            }
            prev = byte[0];
        }

        // Read remaining 17 bytes
        let mut data = [0u8; 17];
        self.port.read_exact(&mut data)?;

        // Checksum: sum of bytes [0..=15], 8-bit wrap, equals data[16]
        let calc: u8 = data[..16].iter().fold(0u8, |acc, v| acc.wrapping_add(*v));
        let checksum_ok = calc == data[16];
        if !checksum_ok {
            warn!(
                "BNO08x RVC checksum mismatch: calc={:#04x}, got={:#04x}",
                calc, data[16]
            );
        }

        // Helpers
        let le_i16 = |lo: u8, hi: u8| i16::from_le_bytes([lo, hi]) as f32;

        // Parse
        let idx = data[0];

        // yaw/pitch/roll: int16 centidegrees -> degrees
        let yaw = le_i16(data[1], data[2]) / 100.0;
        let pitch = le_i16(data[3], data[4]) / 100.0;
        let roll = le_i16(data[5], data[6]) / 100.0;

        // accel: mg -> m/s^2
        const G: f32 = 9.80665;
        let mg_to_ms2 = G / 1000.0;
        let ax = le_i16(data[7], data[8]) * mg_to_ms2;
        let ay = le_i16(data[9], data[10]) * mg_to_ms2;
        let az = le_i16(data[11], data[12]) * mg_to_ms2;

        Ok(RvcPacket {
            index: idx,
            yaw_deg: yaw,
            pitch_deg: pitch,
            roll_deg: roll,
            accel_ms2: [ax, ay, az],
            checksum_ok,
        })
    }

    /// Grab one sample; on error returns last index and NaNs so upstream can keep going.
    fn get_sample(&mut self) -> RvcPacket {
        match self.read_frame() {
            Ok(pkt) => {
                self.last_index = Some(pkt.index);
                pkt
            }
            Err(e) => {
                error!("UART-RVC read error: {e:?}");
                RvcPacket {
                    index: self.last_index.unwrap_or(0),
                    yaw_deg: f32::NAN,
                    pitch_deg: f32::NAN,
                    roll_deg: f32::NAN,
                    accel_ms2: [f32::NAN; 3],
                    checksum_ok: false,
                }
            }
        }
    }

    /// Gravity in SENSOR frame from YPR (degrees), ZYX convention.
    /// yaw doesn't affect gravity; only pitch & roll.
    #[inline]
    fn gravity_from_ypr_deg(ypr_deg: [f32; 3]) -> [f32; 3] {
        let [_yaw, pitch, roll] = ypr_deg;
        let (p, r) = (pitch.to_radians(), roll.to_radians());
        let (sp, cp) = (p.sin(), p.cos());
        let (sr, cr) = (r.sin(), r.cos());
        let g = 9.80665_f32;
        // ZYX → g_body = [-g sin(pitch), g sin(roll) cos(pitch), g cos(roll) cos(pitch)]
        [-g * sp, g * sr * cp, g * cr * cp]
    }

    /// Generate all 48 axis maps (6 permutations × 2^3 sign flips).
    fn all_axis_maps() -> [AxisMap; 48] {
        const PERMS: [[usize; 3]; 6] = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        let mut out: [AxisMap; 48] = [AxisMap {
            perm: [0, 1, 2],
            sign: [1.0, 1.0, 1.0],
        }; 48];
        let mut k = 0usize;
        for p in PERMS {
            for sx in [-1.0, 1.0] {
                for sy in [-1.0, 1.0] {
                    for sz in [-1.0, 1.0] {
                        out[k] = AxisMap {
                            perm: p,
                            sign: [sx, sy, sz],
                        };
                        k += 1;
                    }
                }
            }
        }
        out
    }

    /// Auto-detect axis mapping by minimizing the residual between measured accel (a_raw)
    /// and gravity computed from YPR (g_body), while the unit is stationary.
    fn calibrate_axis_map(&mut self, target_samples: usize, max_duration: Duration) {
        let deadline = Instant::now() + max_duration;

        // Collect samples
        let mut samples: Vec<([f32; 3], [f32; 3])> = Vec::with_capacity(target_samples);
        while samples.len() < target_samples && Instant::now() < deadline {
            match self.read_frame() {
                Ok(pkt) if pkt.yaw_deg.is_finite() && pkt.accel_ms2[0].is_finite() => {
                    let g_body = Self::gravity_from_ypr_deg([pkt.yaw_deg, pkt.pitch_deg, pkt.roll_deg]);
                    samples.push((pkt.accel_ms2, g_body));
                }
                Ok(_) => {} // skip NaNs
                Err(_) => {} // skip timeouts
            }
        }

        if samples.len() < 10 {
            warn!(
                "Axis autodetect: insufficient samples ({}). Using identity map.",
                samples.len()
            );
            self.axis_map = AxisMap {
                perm: [0, 1, 2],
                sign: [1.0, 1.0, 1.0],
            };
            return;
        }

        // Search all 48 maps
        let mut best = AxisMap {
            perm: [0, 1, 2],
            sign: [1.0, 1.0, 1.0],
        };
        let mut best_cost = f32::INFINITY;

        for m in Self::all_axis_maps() {
            let mut sse = 0.0f32;
            for &(a_raw, g_body) in &samples {
                // Map gravity into accel frame, compare to measured accel at rest
                let g_mapped = m.apply(g_body);
                let dx = a_raw[0] - g_mapped[0];
                let dy = a_raw[1] - g_mapped[1];
                let dz = a_raw[2] - g_mapped[2];
                sse += dx * dx + dy * dy + dz * dz;
            }
            if sse < best_cost {
                best_cost = sse;
                best = m;
            }
        }

        self.axis_map = best;
        info!(
            "Axis autodetect chose perm={:?} sign={:?} (cost={:.4}, samples={})",
            best.perm,
            best.sign,
            best_cost / samples.len() as f32,
            samples.len()
        );
    }

    /// Compute linear acceleration in SENSOR frame (m/s^2).
    /// a_lin = a_raw - M * g_body
    fn compute_linear_accel(&self, ypr_deg: [f32; 3], a_raw: [f32; 3]) -> [f32; 3] {
        let g_body = Self::gravity_from_ypr_deg(ypr_deg);
        let g_aligned = self.axis_map.apply(g_body);
        [
            a_raw[0] - g_aligned[0],
            a_raw[1] - g_aligned[1],
            a_raw[2] - g_aligned[2],
        ]
    }
}

impl HardwareSensor for Bno085RvcSensor {
    fn name(&self) -> SensorName {
        SensorName::Bno085
    }

    fn read_data(&mut self) -> SensorData {
        let pkt = self.get_sample();

        let ypr = [pkt.yaw_deg, pkt.pitch_deg, pkt.roll_deg];
        let a_raw = pkt.accel_ms2;

        // Linear acceleration (gravity removed), still in sensor/body frame
        let a_lin = if ypr.iter().all(|v| v.is_finite()) && a_raw.iter().all(|v| v.is_finite()) {
            self.compute_linear_accel(ypr, a_raw)
        } else {
            [f32::NAN; 3]
        };

        SensorData::Bno085 {
            euler_angles: ypr,
            acceleration: a_lin,
        }
    }

    fn read_debug(&mut self) -> String {
        let pkt = self.get_sample();
        let ypr = [pkt.yaw_deg, pkt.pitch_deg, pkt.roll_deg];
        let a_raw = pkt.accel_ms2;

        let g_body = if ypr.iter().all(|v| v.is_finite()) {
            Self::gravity_from_ypr_deg(ypr)
        } else {
            [f32::NAN; 3]
        };

        let g_aligned = self.axis_map.apply(g_body);
        let a_lin = if a_raw.iter().all(|v| v.is_finite()) && g_aligned.iter().all(|v| v.is_finite()) {
            [
                a_raw[0] - g_aligned[0],
                a_raw[1] - g_aligned[1],
                a_raw[2] - g_aligned[2],
            ]
        } else {
            [f32::NAN; 3]
        };

        let norm = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();

        format!(
            "RVC idx:{} | YPR(deg)=({:.2},{:.2},{:.2}) \
             | a_raw=({:.3},{:.3},{:.3}) | |a_raw|={:.3} \
             | g_body=({:.3},{:.3},{:.3}) | g_aligned=({:.3},{:.3},{:.3}) | |g|={:.3} \
             | a_lin=({:.3},{:.3},{:.3}) | |a_lin|={:.3} | csum:{} | map perm={:?} sign={:?}",
            pkt.index,
            ypr[0], ypr[1], ypr[2],
            a_raw[0], a_raw[1], a_raw[2], norm(a_raw),
            g_body[0], g_body[1], g_body[2],
            g_aligned[0], g_aligned[1], g_aligned[2], norm(g_aligned),
            a_lin[0], a_lin[1], a_lin[2], norm(a_lin),
            if pkt.checksum_ok { "ok" } else { "BAD" },
            self.axis_map.perm, self.axis_map.sign
        )
    }

    fn end_calibration(&mut self) -> anyhow::Result<()> {
        // No host-side calibration save in RVC mode; sensor streams continuously.
        info!("BNO085 RVC: end_calibration() is a no-op in UART-RVC mode");
        Ok(())
    }
}
