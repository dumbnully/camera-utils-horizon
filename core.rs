use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub key: &'static str,
    pub default: f64,
    pub min: f64,
    pub max: f64,
}
pub const SPECS: &[Spec] = &[
    Spec {
        key: "smoothness",
        default: 0.0,
        min: 0.0,
        max: 1.0,
    },
    Spec {
        key: "min_smoothness",
        default: 40.0,
        min: 1.0,
        max: 1000.0,
    },
    Spec {
        key: "max_smoothness",
        default: 100.0,
        min: 1.0,
        max: 1000.0,
    },
    Spec {
        key: "cinematic_camera_modifier",
        default: 1.0,
        min: 0.0,
        max: 1.0,
    },
    Spec {
        key: "zoom",
        default: 0.1,
        min: 0.0,
        max: 2.0,
    },
    Spec {
        key: "zoom_sensitivity",
        default: 0.01,
        min: 0.001,
        max: 1.0,
    },
    Spec {
        key: "third_person_offset",
        default: 0.0,
        min: -32.0,
        max: 32.0,
    },
    Spec {
        key: "third_person_offset_sensitivity",
        default: 0.1,
        min: 0.001,
        max: 1.0,
    },
    Spec {
        key: "third_person_cam_1_offset_x",
        default: -4.0,
        min: -100.0,
        max: 100.0,
    },
    Spec {
        key: "third_person_cam_1_offset_y",
        default: 0.0,
        min: -100.0,
        max: 100.0,
    },
    Spec {
        key: "third_person_cam_1_offset_z",
        default: 0.0,
        min: -100.0,
        max: 100.0,
    },
    Spec {
        key: "third_person_cam_1_rotation_x",
        default: 0.0,
        min: -90.0,
        max: 90.0,
    },
    Spec {
        key: "third_person_cam_2_offset_x",
        default: -4.0,
        min: -100.0,
        max: 100.0,
    },
    Spec {
        key: "third_person_cam_2_offset_y",
        default: 0.0,
        min: -100.0,
        max: 100.0,
    },
    Spec {
        key: "third_person_cam_2_offset_z",
        default: 0.0,
        min: -100.0,
        max: 100.0,
    },
    Spec {
        key: "third_person_cam_2_rotation_x",
        default: 0.0,
        min: -90.0,
        max: 90.0,
    },
    Spec {
        key: "gui_opacity",
        default: 1.0,
        min: 0.0,
        max: 1.0,
    },
    Spec {
        key: "zoom_animation_from",
        default: 1.0,
        min: 0.01,
        max: 2.0,
    },
    Spec {
        key: "zoom_animation_to",
        default: 0.1,
        min: 0.01,
        max: 2.0,
    },
    Spec {
        key: "zoom_animation_duration",
        default: 200.0,
        min: 1.0,
        max: 2147483647.0,
    },
    Spec {
        key: "third_person_cam_1_inverted",
        default: 0.0,
        min: 0.0,
        max: 1.0,
    },
    Spec {
        key: "third_person_cam_1_hide_gui",
        default: 0.0,
        min: 0.0,
        max: 1.0,
    },
    Spec {
        key: "third_person_cam_2_inverted",
        default: 0.0,
        min: 0.0,
        max: 1.0,
    },
    Spec {
        key: "third_person_cam_2_hide_gui",
        default: 0.0,
        min: 0.0,
        max: 1.0,
    },
    Spec {
        key: "modifier_key",
        default: 3.0,
        min: 0.0,
        max: 8.0,
    },
];
pub const MODIFIERS: &[&str] = &[
    "LEFT_CTRL",
    "RIGHT_CTRL",
    "CTRL",
    "RIGHT_ALT",
    "LEFT_ALT",
    "ALT",
    "LEFT_SHIFT",
    "RIGHT_SHIFT",
    "SHIFT",
];
#[derive(Clone, Debug)]
pub struct Config(pub BTreeMap<String, f64>);
impl Default for Config {
    fn default() -> Self {
        Self(
            SPECS
                .iter()
                .map(|s| (s.key.to_owned(), s.default))
                .collect(),
        )
    }
}
impl Config {
    pub fn get(&self, key: &str) -> f64 {
        self.0.get(key).copied().unwrap_or(0.0)
    }
    pub fn set(&mut self, key: &str, value: f64) -> bool {
        let Some(s) = SPECS.iter().find(|s| s.key == key) else {
            return false;
        };
        if !value.is_finite() {
            return false;
        }
        let mut v = value.clamp(s.min, s.max);
        if s.key == "zoom_animation_duration"
            || s.key == "modifier_key"
            || s.max == 1.0 && s.min == 0.0 && s.key.contains("cam_")
        {
            v = v.round();
        }
        self.0.insert(key.into(), v);
        true
    }
    pub fn parse(text: &str) -> Self {
        let mut c = Self::default();
        for line in text
            .lines()
            .filter(|l| !l.trim_start().starts_with(['#', '!']))
        {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let (k, v) = (k.trim(), v.trim());
            let value = if k == "modifier_key" {
                MODIFIERS
                    .iter()
                    .position(|m| *m == v)
                    .map(|i| i as f64)
                    .or_else(|| v.parse().ok())
            } else {
                match v {
                    "true" => Some(1.0),
                    "false" => Some(0.0),
                    _ => v.parse().ok(),
                }
            };
            if let Some(v) = value {
                c.set(k, v);
            }
        }
        c
    }
    pub fn encode(&self) -> String {
        let mut text = String::new();
        for s in SPECS {
            let value = self.get(s.key);
            let value = if s.key == "modifier_key" {
                MODIFIERS[value as usize].into()
            } else if s.key.ends_with("inverted") || s.key.ends_with("hide_gui") {
                (value != 0.0).to_string()
            } else {
                value.to_string()
            };
            text.push_str(&format!("{}={}\n", s.key, value));
        }
        text
    }
    pub fn reset(&mut self, keys: &[&str]) {
        for key in keys {
            if let Some(s) = SPECS.iter().find(|s| s.key == *key) {
                self.set(key, s.default);
            }
        }
    }
}
pub fn zoom_factor(value: f64) -> f32 {
    if value >= 2.0 {
        1.5
    } else if value <= 0.0 {
        0.1
    } else if value > 1.0 {
        (1.0 + (value - 1.0) * 0.5) as f32
    } else if value < 1.0 {
        (0.1 + value * 0.9) as f32
    } else {
        1.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ZoomTrack {
    pub from: f32,
    pub to: f32,
    pub duration: u32,
    pub time: u32,
}
impl ZoomTrack {
    pub fn tick(&mut self) {
        self.time = self.time.saturating_add(1);
    }
    pub fn value(self) -> f32 {
        let t = (self.time as f32 / self.duration.max(1) as f32).clamp(0.0, 1.0);
        self.from + (self.to - self.from) * t
    }
}

pub fn camera_move(eye: [f64; 3], yaw: f32, pitch: f32, offset: [f64; 3]) -> [f64; 3] {
    let (sy, cy) = f64::from(yaw).to_radians().sin_cos();
    let (sp, cp) = f64::from(pitch).to_radians().sin_cos();
    let [f, u, l] = offset;
    [
        eye[0] - sy * cp * f - sy * sp * u - cy * l,
        eye[1] - sp * f + cp * u,
        eye[2] + cy * cp * f + cy * sp * u - sy * l,
    ]
}
#[derive(Default, Debug)]
pub struct Smooth {
    target: f64,
    remaining: f64,
    last: f64,
}
impl Smooth {
    pub fn step(&mut self, delta: f64, time: f64) -> f64 {
        self.target += delta;
        let mut d = self.target - self.remaining;
        let last = self.last + 0.5 * (d - self.last);
        let sign = if d == 0.0 || d.is_nan() {
            d
        } else {
            d.signum()
        };
        if sign * d > sign * self.last {
            d = last;
        }
        self.last = last;
        self.remaining += d * time;
        d * time
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_zoom_curve() {
        for (v, want) in [(0., 0.1), (0.1, 0.19), (1., 1.), (1.5, 1.25), (2., 1.5)] {
            assert!((zoom_factor(v) - want).abs() < 1e-6);
        }
    }
    #[test]
    fn animation_is_ticks_and_holds_endpoint() {
        let mut t = ZoomTrack {
            from: 1.,
            to: 0.1,
            duration: 200,
            time: 0,
        };
        assert_eq!(t.value(), 1.);
        for _ in 0..100 {
            t.tick();
        }
        assert!((t.value() - 0.55).abs() < 1e-6);
        for _ in 0..300 {
            t.tick();
        }
        assert!((t.value() - 0.1).abs() < 1e-6);
    }
    #[test]
    fn property_roundtrip_and_limits() {
        let mut c = Config::parse(
            "zoom=NaN\nthird_person_cam_1_inverted=true\nzoom_animation_duration=0\nmodifier_key=RIGHT_ALT\nthird_person_offset=999",
        );
        assert_eq!(c.get("zoom"), 0.1);
        assert_eq!(c.get("zoom_animation_duration"), 1.);
        assert_eq!(c.get("third_person_offset"), 32.);
        assert_eq!(c.get("third_person_cam_1_inverted"), 1.);
        c.set("zoom", 0.4);
        assert_eq!(Config::parse(&c.encode()).0, c.0);
    }
    #[test]
    fn camera_axes_match_minecraft() {
        assert_eq!(camera_move([0.; 3], 0., 0., [-4., 0., 0.]), [0., 0., -4.]);
        let p = camera_move([1., 2., 3.], 90., 0., [-4., 0., 0.]);
        assert!((p[0] - 5.).abs() < 1e-6);
        assert!((p[2] - 3.).abs() < 1e-6);
    }
    #[test]
    fn smoothing_preserves_zero_after_negative_motion() {
        let mut s = Smooth::default();
        assert_eq!(s.step(-1.0, 2.0), -1.0);
        assert_eq!(s.step(0.0, 0.1), 0.0);
    }
    #[test]
    fn zero_offset_keeps_eye() {
        assert_eq!(camera_move([1., 2., 3.], 42., -28., [0.; 3]), [1., 2., 3.]);
    }
}
