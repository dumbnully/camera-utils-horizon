use super::camera_utils_core::{Config, MODIFIERS, SPECS, Smooth, ZoomTrack, zoom_factor};
use super::{Game, Screen, perspective::CameraType};
use horizon_client::{
    hud::HudBatch,
    mod_api::{NativeMod, NativeScreenView},
};
use std::{cell::RefCell, collections::HashSet, path::PathBuf};
pub const ID: &str = "camerautils";
#[derive(Default)]
struct State {
    config: Config,
    path: Option<PathBuf>,
    held: HashSet<String>,
    profile: Option<usize>,
    detached: bool,
    frozen: Option<([f64; 3], f32, f32, String)>,
    track: Option<ZoomTrack>,
    page: usize,
    drag: Option<usize>,
    selected: usize,
    keyboard_focus: bool,
    close_requested: bool,
    sx: Smooth,
    sy: Smooth,
    last_mouse_time: Option<f64>,
    saved_hud: Option<bool>,
    status: Option<(String, u32)>,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}
fn enabled(game: &Game<'_>) -> bool {
    game.native_on(ID) && game.native_window()
}
fn init(game: &Game<'_>, s: &mut State) {
    if s.path.is_some() {
        return;
    }
    let Ok(dir) = Game::native_config_dir() else {
        return;
    };
    let path = dir.join(ID).join("camerautils.properties");
    if let Ok(text) = std::fs::read_to_string(&path) {
        s.config = Config::parse(&text);
    }
    s.path = Some(path);
    save(s);
    let _ = game;
}
fn save(s: &State) {
    if let Some(path) = &s.path {
        let result = (|| -> std::io::Result<()> {
            std::fs::create_dir_all(path.parent().unwrap())?;
            let tmp = path.with_extension("properties.tmp");
            std::fs::write(&tmp, s.config.encode())?;
            std::fs::rename(tmp, path)
        })();
        if let Err(e) = result {
            eprintln!("[Camera Utils] {}: {e}", path.display());
        }
    }
}
fn text<'a>(game: &Game<'_>, label: &'a str) -> &'a str {
    super::camera_utils_i18n::tr(&game.settings.language, label)
}
const KEYS: &[(&str, &str, &str)] = &[
    ("zoom", "Zoom (hold)", "key.keyboard.z"),
    ("third_person_cam_1", "Camera 1", "key.keyboard.f6"),
    ("third_person_cam_2", "Camera 2", "key.keyboard.f7"),
    (
        "third_person_offset",
        "Third-person distance (hold + wheel)",
        "key.keyboard.unknown",
    ),
    ("detach_camera", "Detach camera", "key.keyboard.unknown"),
    ("zoom_animation", "Zoom animation", "key.keyboard.unknown"),
    (
        "cinematic_camera_settings",
        "Cinematic camera settings",
        "key.keyboard.unknown",
    ),
    (
        "third_person_camera_1_settings",
        "Camera 1 settings",
        "key.keyboard.unknown",
    ),
    (
        "third_person_camera_2_settings",
        "Camera 2 settings",
        "key.keyboard.unknown",
    ),
    (
        "third_person_settings",
        "Third-person settings",
        "key.keyboard.unknown",
    ),
    ("zoom_settings", "Zoom settings", "key.keyboard.unknown"),
    (
        "zoom_animation_settings",
        "Zoom animation settings",
        "key.keyboard.unknown",
    ),
];
fn binding(game: &Game<'_>, name: &str) -> String {
    let default = KEYS
        .iter()
        .find(|k| k.0 == name)
        .map_or("key.keyboard.unknown", |k| k.2);
    game.native_mod_key(ID, name, default)
}
fn down(game: &Game<'_>, s: &State, name: &str) -> bool {
    s.held.contains(&binding(game, name))
}
fn modifier(s: &State) -> bool {
    let has = |key: &str| s.held.contains(&format!("key.keyboard.{key}"));
    match s.config.get("modifier_key") as usize {
        0 => has("left.control"),
        1 => has("right.control"),
        2 => has("left.control") || has("right.control"),
        3 => has("right.alt"),
        4 => has("left.alt"),
        5 => has("left.alt") || has("right.alt"),
        6 => has("left.shift"),
        7 => has("right.shift"),
        _ => has("left.shift") || has("right.shift"),
    }
}
pub(super) fn mods(game: &mut Game<'_>, mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: ID.into(),
        title: "Camera Utils 1.1.1".into(),
        description: text(game, "Camera controls by Max Henkel").into(),
        keys: KEYS
            .iter()
            .map(|(name, label, default)| {
                [(*name).into(), text(game, label).into(), (*default).into()]
            })
            .collect(),
    });
}
fn update_hud(game: &mut Game<'_>, s: &mut State) {
    let hide = s.profile.is_some_and(|p| {
        s.config
            .get(&format!("third_person_cam_{}_hide_gui", p + 1))
            != 0.0
    });
    if hide {
        if s.saved_hud.is_none() {
            s.saved_hud = Some(game.ui.hide_gui);
        }
        game.ui.hide_gui = true;
    } else if let Some(hidden) = s.saved_hud.take() {
        game.ui.hide_gui = hidden;
    }
}
fn open(game: &mut Game<'_>, s: &mut State, page: usize) {
    if game.native_mod_screen_open(ID) {
        s.page = page;
        s.drag = None;
        s.selected = 0;
        s.held
            .retain(|k| k.contains("alt") || k.contains("control") || k.contains("shift"));
    }
}
pub(super) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !enabled(game) {
        return;
    }
    with(|s| {
        init(game, s);
        if pressed {
            s.held.insert(key.into());
        } else {
            s.held.remove(key);
            return;
        }
        if !game.native_playing() {
            return;
        }
        let mod_down = modifier(s);
        for (name, page) in [
            ("cinematic_camera_settings", 1),
            ("third_person_camera_1_settings", 2),
            ("third_person_camera_2_settings", 3),
            ("third_person_settings", 4),
            ("zoom_settings", 5),
            ("zoom_animation_settings", 6),
        ] {
            if key == binding(game, name) {
                open(game, s, page);
                return;
            }
        }
        if key == game.settings.keys.get("key.smoothCamera") && mod_down {
            open(game, s, 1);
            return;
        }
        if key == binding(game, "zoom") && mod_down {
            open(game, s, 5);
            return;
        }
        if key == binding(game, "third_person_offset") && mod_down {
            open(game, s, 4);
            return;
        }
        for p in 0..2 {
            if key
                == binding(
                    game,
                    if p == 0 {
                        "third_person_cam_1"
                    } else {
                        "third_person_cam_2"
                    },
                )
            {
                if mod_down {
                    open(game, s, p + 2);
                    return;
                }
                if s.profile == Some(p) {
                    s.profile = None;
                    game.camera_type = CameraType::FirstPerson;
                } else {
                    s.profile = Some(p);
                    s.detached = false;
                    game.camera_type = CameraType::ThirdPersonBack;
                }
                update_hud(game, s);
                game.check_entity_post_effect(game.camera_type.first_person());
                return;
            }
        }
        if key == binding(game, "detach_camera") {
            s.detached = !s.detached;
            if s.detached {
                s.profile = None;
                let eye = game.player.as_ref().map(|p| {
                    [
                        p.position.x,
                        p.position.y + f64::from(p.dimensions.eye_height),
                        p.position.z,
                    ]
                });
                if let Some(eye) = eye {
                    let reuse = mod_down
                        && s.frozen.as_ref().is_some_and(|(at, _, _, dim)| {
                            dim == game.dim.id()
                                && at
                                    .iter()
                                    .zip(eye)
                                    .map(|(a, b)| (a - b).powi(2))
                                    .sum::<f64>()
                                    <= 10000.0
                        });
                    if !reuse {
                        s.frozen = Some((eye, game.yaw, game.pitch, game.dim.id().into()));
                    }
                }
                game.camera_type = CameraType::ThirdPersonBack;
            } else {
                game.camera_type = CameraType::FirstPerson;
            }
            update_hud(game, s);
            game.check_entity_post_effect(game.camera_type.first_person());
            return;
        }
        if key == binding(game, "zoom_animation") {
            if mod_down {
                open(game, s, 6);
            } else if s.track.is_some() {
                s.track = None;
            } else {
                s.track = Some(ZoomTrack {
                    from: s.config.get("zoom_animation_from") as f32,
                    to: s.config.get("zoom_animation_to") as f32,
                    duration: s.config.get("zoom_animation_duration") as u32,
                    time: 0,
                });
            }
        }
    });
}
pub(super) fn cycle(game: &mut Game<'_>) {
    with(|s| {
        if s.profile.is_some() {
            game.camera_type = CameraType::FirstPerson;
        }
        s.profile = None;
        s.detached = false;
        update_hud(game, s);
    });
}
pub(super) fn tick(game: &mut Game<'_>) {
    if !game.native_window() {
        return;
    }
    with(|s| {
        init(game, s);
        if s.close_requested {
            s.close_requested = false;
            game.native_mod_screen_close(ID);
        }
        if !enabled(game) || game.player.is_none() || game.title.is_some() {
            if s.profile.is_some() || s.detached {
                game.camera_type = CameraType::FirstPerson;
            }
            s.profile = None;
            s.detached = false;
            s.track = None;
            s.held.clear();
            s.frozen = None;
            update_hud(game, s);
            return;
        }
        if let Some(t) = &mut s.track {
            t.tick();
        }
        if !game.focused && !game.agent_pointer {
            s.held.clear();
        }
        if let Some((_, left)) = &mut s.status {
            *left = left.saturating_sub(1);
            if *left == 0 {
                s.status = None;
            }
        }
        if s.frozen
            .as_ref()
            .is_some_and(|(_, _, _, dim)| dim != game.dim.id())
        {
            s.detached = false;
            s.frozen = None;
            game.camera_type = CameraType::FirstPerson;
        }
        update_hud(game, s);
    });
}
pub(super) fn wheel(game: &mut Game<'_>, steps: f32, claimed: &mut bool) {
    if !enabled(game) || !game.native_playing() {
        return;
    }
    with(|s| {
        init(game, s);
        if down(game, s, "third_person_offset")
            && !game.camera_type.first_person()
            && s.profile.is_none()
        {
            let v = (s.config.get("third_person_offset")
                - f64::from(steps) * s.config.get("third_person_offset_sensitivity"))
            .clamp(-32., 32.);
            s.config.set("third_person_offset", v);
            s.status = Some((
                format!("{}: {v:.2}", text(game, "Camera distance offset")),
                40,
            ));
        } else if down(game, s, "zoom") {
            let v = (s.config.get("zoom") - f64::from(steps) * s.config.get("zoom_sensitivity"))
                .clamp(0., 2.);
            s.config.set("zoom", v);
            s.status = Some((
                format!("{}: {:.0}%", text(game, "Zoom"), (1. - v) * 100.),
                40,
            ));
        } else {
            return;
        }
        *claimed = true;
        save(s);
    });
}
pub(super) fn fov(game: &Game<'_>, base: f32, modifier_out: &mut f32) {
    if !enabled(game) {
        return;
    }
    with(|s| {
        init(game, s);
        *modifier_out = if let Some(t) = s.track {
            t.value()
        } else {
            base * if game.native_playing() && down(game, s, "zoom") {
                zoom_factor(s.config.get("zoom"))
            } else {
                1.
            }
        };
    });
}
pub(super) fn distance(
    game: &Game<'_>,
    entity: &horizon_client::camera::CameraEntity,
    distance: &mut f32,
) {
    if !enabled(game) {
        return;
    }
    with(|s| {
        init(game, s);
        let offset = s.config.get("third_person_offset");
        let (scale, base, adjusted) = entity.living.as_ref().map_or((1.0, 4.0, 4.0), |living| {
            (
                living.scale,
                living.camera_distance as f32,
                (living.camera_distance + offset).clamp(0.0, 32.0) as f32,
            )
        });
        let (mount_scale, mount_base, mount_adjusted) =
            entity
                .mount
                .map_or((scale, base, adjusted), |(scale, distance)| {
                    (
                        scale,
                        distance as f32,
                        (distance + offset).clamp(0.0, 32.0) as f32,
                    )
                });
        let vanilla = (scale * base).max(mount_scale * mount_base);
        let modified = (scale * adjusted).max(mount_scale * mount_adjusted);
        *distance = (*distance + modified - vanilla).clamp(0.0, 1024.0);
    });
}

pub(super) fn pose(game: &Game<'_>, view: &mut horizon_client::camera::Camera) {
    if !enabled(game) {
        return;
    }
    with(|s| {
        init(game, s);
        if s.detached {
            view.detached = true;
            if let Some((at, yaw, pitch, _)) = &s.frozen {
                view.position = *at;
                view.set_rotation(*yaw, *pitch);
            }
        } else if let Some(p) = s.profile {
            view.detached = true;
            let prefix = format!("third_person_cam_{}_", p + 1);
            let offset = [
                s.config.get(&(prefix.clone() + "offset_x")),
                s.config.get(&(prefix.clone() + "offset_y")),
                s.config.get(&(prefix.clone() + "offset_z")),
            ];
            view.position = view.eye;
            view.move_by(offset[0] as f32, offset[1] as f32, offset[2] as f32);
            let invert = s.config.get(&(prefix.clone() + "inverted")) != 0.;
            view.x_rot = s.config.get(&(prefix + "rotation_x")) as f32
                + if invert { -view.x_rot } else { view.x_rot };
            view.set_rotation(view.y_rot + if invert { 180. } else { 0. }, view.x_rot);
        }
    });
}
pub(super) fn special(game: &Game<'_>) -> bool {
    enabled(game) && with(|s| s.profile.is_some() || s.detached)
}
pub(super) fn detached(game: &Game<'_>) -> bool {
    enabled(game) && with(|s| s.detached)
}
pub(super) fn hud_layer(game: &Game<'_>, layer: &str, canceled: &mut bool) {
    if enabled(game) && with(|s| s.profile.is_some()) && layer == "minecraft:crosshair" {
        *canceled = true;
    }
}
pub(super) fn hud(game: &Game<'_>, batch: &mut HudBatch) {
    if !enabled(game) {
        return;
    }
    with(|s| {
        if let Some((text, _)) = &s.status {
            let x = (batch.width - game.native_text_width(text)) * 0.5;
            game.native_text(batch, text, [x, batch.height - 65.], 0xffffff, true);
        }
    });
}

pub(super) fn mouse(game: &mut Game<'_>) -> bool {
    if !enabled(game) {
        return false;
    }
    with(|s| {
        init(game, s);
        let time = game.started.elapsed().as_secs_f64();
        let dt = s
            .last_mouse_time
            .replace(time)
            .map_or(0., |last| time - last);
        if !game.native_playing()
            || !(game.focused || game.agent_pointer)
            || !(game.captured || game.agent_pointer)
        {
            s.sx = Smooth::default();
            s.sy = Smooth::default();
            return false;
        }
        let mut look = game.look_options();
        let zoom = down(game, s, "zoom");
        if zoom {
            look.sensitivity *= s.config.get("zoom").min(1.);
        }
        let smooth = game.smooth_camera || s.config.get("smoothness") > 0.;
        if !zoom && !smooth {
            s.sx = Smooth::default();
            s.sy = Smooth::default();
            return false;
        }
        let (dx, dy) = game.mouse.accumulated();

        game.mouse
            .handle_accumulated_movement(time, false, false, look);
        let ss = look.sensitivity * f64::from(0.6f32) + f64::from(0.2f32);
        let sensitivity_mod = ss * ss * ss;
        let sens = sensitivity_mod * 8.;
        let (mut xo, mut yo) = if smooth {
            let multiplier = if game.smooth_camera {
                1. + 99. * (1. - s.config.get("cinematic_camera_modifier"))
            } else {
                s.config.get("min_smoothness")
                    + (s.config.get("max_smoothness") - s.config.get("min_smoothness"))
                        * (1. - s.config.get("smoothness"))
            };
            (
                s.sx.step(dx * sens, dt * sens * multiplier),
                s.sy.step(dy * sens, dt * sens * multiplier),
            )
        } else {
            s.sx = Smooth::default();
            s.sy = Smooth::default();
            let scale = if look.scoping { sensitivity_mod } else { sens };
            (dx * scale, dy * scale)
        };
        if look.invert_x {
            xo = -xo;
        }
        if look.invert_y {
            yo = -yo;
        }
        game.tutorial.mouse(xo, yo);
        let scale = game.zoom_turn_scale();
        game.turn_player(xo * scale, yo * scale);
        true
    })
}
#[derive(Clone, Copy)]
struct Row {
    key: &'static str,
    label: &'static str,

    min: f64,
    max: f64,
    step: f64,
}
const R: &[Row] = &[
    Row {
        key: "cinematic_camera_modifier",
        label: "Cinematic modifier",

        min: 0.,
        max: 1.,
        step: 0.05,
    },
    Row {
        key: "smoothness",
        label: "Always-on smoothing",

        min: 0.,
        max: 1.,
        step: 0.01,
    },
    Row {
        key: "third_person_cam_1_offset_x",
        label: "Forward / back",

        min: -10.,
        max: 10.,
        step: 0.1,
    },
    Row {
        key: "third_person_cam_1_offset_y",
        label: "Vertical offset",

        min: -5.,
        max: 5.,
        step: 0.1,
    },
    Row {
        key: "third_person_cam_1_offset_z",
        label: "Sideways offset",

        min: -5.,
        max: 5.,
        step: 0.1,
    },
    Row {
        key: "third_person_cam_1_rotation_x",
        label: "Pitch offset",

        min: -90.,
        max: 90.,
        step: 1.,
    },
    Row {
        key: "third_person_cam_1_inverted",
        label: "Inverted",

        min: 0.,
        max: 1.,
        step: 1.,
    },
    Row {
        key: "third_person_cam_1_hide_gui",
        label: "Hide HUD",

        min: 0.,
        max: 1.,
        step: 1.,
    },
    Row {
        key: "third_person_cam_2_offset_x",
        label: "Forward / back",

        min: -10.,
        max: 10.,
        step: 0.1,
    },
    Row {
        key: "third_person_cam_2_offset_y",
        label: "Vertical offset",

        min: -5.,
        max: 5.,
        step: 0.1,
    },
    Row {
        key: "third_person_cam_2_offset_z",
        label: "Sideways offset",

        min: -5.,
        max: 5.,
        step: 0.1,
    },
    Row {
        key: "third_person_cam_2_rotation_x",
        label: "Pitch offset",

        min: -90.,
        max: 90.,
        step: 1.,
    },
    Row {
        key: "third_person_cam_2_inverted",
        label: "Inverted",

        min: 0.,
        max: 1.,
        step: 1.,
    },
    Row {
        key: "third_person_cam_2_hide_gui",
        label: "Hide HUD",

        min: 0.,
        max: 1.,
        step: 1.,
    },
    Row {
        key: "third_person_offset_sensitivity",
        label: "Wheel sensitivity",

        min: 0.1,
        max: 1.,
        step: 0.1,
    },
    Row {
        key: "third_person_offset",
        label: "Distance offset",

        min: -32.,
        max: 32.,
        step: 0.25,
    },
    Row {
        key: "zoom_sensitivity",
        label: "Wheel sensitivity",

        min: 0.01,
        max: 0.25,
        step: 0.01,
    },
    Row {
        key: "zoom",
        label: "Zoom",

        min: 0.,
        max: 2.,
        step: 0.01,
    },
    Row {
        key: "zoom_animation_from",
        label: "Starting FOV scale",

        min: 0.01,
        max: 2.,
        step: 0.01,
    },
    Row {
        key: "zoom_animation_to",
        label: "Ending FOV scale",

        min: 0.01,
        max: 2.,
        step: 0.01,
    },
    Row {
        key: "zoom_animation_duration",
        label: "Duration (ticks)",

        min: 1.,
        max: 600.,
        step: 1.,
    },
    Row {
        key: "min_smoothness",
        label: "Minimum smoothing",

        min: 1.,
        max: 1000.,
        step: 1.,
    },
    Row {
        key: "max_smoothness",
        label: "Maximum smoothing",

        min: 1.,
        max: 1000.,
        step: 1.,
    },
    Row {
        key: "modifier_key",
        label: "Settings modifier",

        min: 0.,
        max: 8.,
        step: 1.,
    },
];
const PAGES: &[&str] = &[
    "Camera Utils",
    "Cinematic camera",
    "Third-person camera 1",
    "Third-person camera 2",
    "Third-person distance",
    "Zoom",
    "Zoom animation",
    "Advanced settings",
];
fn rows(page: usize) -> &'static [Row] {
    match page {
        1 => &R[0..2],
        2 => &R[2..8],
        3 => &R[8..14],
        4 => &R[14..16],
        5 => &R[16..18],
        6 => &R[18..21],
        7 => &R[21..24],
        _ => &R[0..0],
    }
}
fn boolean(row: Row) -> bool {
    row.key.ends_with("inverted") || row.key.ends_with("hide_gui")
}
fn label(game: &Game<'_>, s: &State, row: Row) -> String {
    let v = s.config.get(row.key);
    let value = if boolean(row) {
        if v != 0. {
            text(game, "On")
        } else {
            text(game, "Off")
        }
        .into()
    } else if row.key == "modifier_key" {
        let names = [
            "key.keyboard.left.control",
            "key.keyboard.right.control",
            "key.keyboard.left.control",
            "key.keyboard.right.alt",
            "key.keyboard.left.alt",
            "key.keyboard.left.alt",
            "key.keyboard.left.shift",
            "key.keyboard.right.shift",
            "key.keyboard.left.shift",
        ];
        horizon_client::keybinds::display_name(names[v as usize]).plain()
    } else if row.key == "zoom_animation_duration" {
        format!("{v:.0} {} ({:.2}s)", text(game, "ticks"), v / 20.)
    } else if row.key == "zoom" {
        format!("{:.0}%", (1. - v) * 100.)
    } else if row.key.contains("offset") && !row.key.contains("sensitivity") {
        format!("{v:.2} {}", text(game, "Blocks"))
    } else if row.key.contains("rotation") {
        format!("{v:.0}\u{00b0}")
    } else if row.key.starts_with("zoom_animation_") {
        format!("{:.0}%", v * 100.)
    } else {
        format!("{v:.2}")
    };
    format!("{}: {value}", text(game, row.label))
}
fn layout(width: f32, height: f32, count: usize) -> (f32, f32, f32, f32) {
    let w = 248_f32.min((width - 8.).max(100.));
    let step = 25_f32.min(((height - 45.) / (count + 1) as f32).max(12.));
    let h = 35. + step * (count + 1) as f32;
    ((width - w) * 0.5, ((height - h) * 0.5).max(2.), w, step)
}
fn row_rect(x: f32, y: f32, w: f32, step: f32, i: usize) -> [f32; 4] {
    [x + 10., y + 25. + i as f32 * step, w - 20., step - 5.]
}
fn inside(p: [f32; 2], r: [f32; 4]) -> bool {
    p[0] >= r[0] && p[0] < r[0] + r[2] && p[1] >= r[1] && p[1] < r[1] + r[3]
}
fn set_slider(s: &mut State, row: Row, t: f64) {
    let v = row.min + (row.max - row.min) * t.clamp(0., 1.);
    s.config.set(row.key, (v / row.step).round() * row.step);
    save(s);
}

fn button(game: &Game<'_>, batch: &mut HudBatch, r: [f32; 4], text: &str, hover: bool) {
    horizon_client::ui::button(
        batch,
        &game.gui.atlas,
        r[0],
        r[1],
        r[2],
        r[3],
        &horizon_client::text::Text::literal(text),
        true,
        hover,
    );
}
fn panel(game: &Game<'_>, batch: &mut HudBatch, r: [f32; 4], opacity: f32) {
    let [x, y, w, h] = r;
    let alpha = (opacity * 255.) as u32;
    for (rect, rgb) in [
        ([x, y, w, h], 0x101010),
        ([x + 1., y + 1., w - 2., h - 2.], 0x555555),
        ([x + 1., y + 1., w - 3., h - 3.], 0xffffff),
        ([x + 3., y + 3., w - 6., h - 6.], 0xc6c6c6),
    ] {
        game.native_fill(batch, rect, (alpha << 24) | rgb);
    }
}
fn eye(game: &Game<'_>, batch: &mut HudBatch, x: f32, y: f32, hover: bool) {
    if hover {
        game.native_fill(batch, [x - 2., y - 2., 16., 14.], 0xffa0a0a0);
    }
    for (r, c) in [
        ([x + 3., y, 6., 1.], 0xff303030),
        ([x + 1., y + 1., 10., 1.], 0xff303030),
        ([x, y + 2., 12., 5.], 0xff303030),
        ([x + 1., y + 7., 10., 1.], 0xff303030),
        ([x + 3., y + 8., 6., 1.], 0xff303030),
        ([x + 1., y + 3., 10., 3.], 0xffeeeeee),
        ([x + 3., y + 2., 6., 5.], 0xffeeeeee),
        ([x + 5., y + 2., 3., 5.], 0xff202020),
        ([x + 5., y + 2., 1., 1.], 0xffffffff),
    ] {
        game.native_fill(batch, r, c);
    }
}
pub(super) fn draw(game: &Game<'_>, batch: &mut HudBatch, _screen: &NativeScreenView<'_>) {
    if !enabled(game) || game.native_mod_screen() != Some(ID) {
        return;
    }
    with(|s| {
        init(game, s);
        let rs = rows(s.page);
        let n = if s.page == 0 {
            PAGES.len() - 1
        } else {
            rs.len() + 1
        };
        let (x, y, w, step) = layout(batch.width, batch.height, n);
        panel(
            game,
            batch,
            [x, y, w, 35. + step * (n + 1) as f32],
            s.config.get("gui_opacity") as f32,
        );
        let title = text(game, PAGES[s.page]);
        horizon_client::ui::scrolling_text(
            batch,
            &game.gui.atlas,
            &horizon_client::text::Text::literal(title),
            x + 8.,
            x + w - 26.,
            y + 4.,
            y + 22.,
            y + 8.,
            0xff404040,
        );
        eye(
            game,
            batch,
            x + w - 20.,
            y + 6.,
            inside(game.native_cursor(), [x + w - 23., y + 3., 20., 18.]),
        );
        for i in 0..n {
            let r = row_rect(x, y, w, step, i);
            let hover = inside(game.native_cursor(), r);
            let focused = s.keyboard_focus && s.selected == i;
            if s.page != 0 && i < rs.len() && !boolean(rs[i]) && rs[i].key != "modifier_key" {
                let row = rs[i];
                let t =
                    ((s.config.get(row.key) - row.min) / (row.max - row.min)).clamp(0., 1.) as f32;
                let start = batch.vertices.len();
                horizon_client::ui::slider_ext(
                    batch,
                    &game.gui.atlas,
                    r[0],
                    r[1],
                    r[2],
                    &horizon_client::text::Text::literal(label(game, s, row)),
                    t,
                    hover,
                    focused,
                    focused,
                    true,
                    s.drag == Some(i),
                );
                if r[3] != 20. {
                    batch.scale_since(start, r[0], r[1], 1., r[3] / 20.);
                }
            } else {
                let text = if s.page == 0 {
                    text(game, PAGES[i + 1]).to_owned()
                } else if i == rs.len() {
                    text(game, "Reset this page").into()
                } else {
                    label(game, s, rs[i])
                };
                button(game, batch, r, &text, hover || focused);
            }
        }
        let r = row_rect(x, y, w, step, n);
        let text = text(game, if s.page == 0 { "Close (Esc)" } else { "Back" });
        button(
            game,
            batch,
            r,
            text,
            inside(game.native_cursor(), r) || (s.keyboard_focus && s.selected == n),
        );
    });
}
pub(super) fn click(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    if !enabled(game) || game.native_mod_screen() != Some(ID) || button != 0 {
        return;
    }
    *canceled = true;
    with(|s| {
        s.keyboard_focus = false;
        let rs = rows(s.page);
        let n = if s.page == 0 {
            PAGES.len() - 1
        } else {
            rs.len() + 1
        };
        let (x, y, w, step) = layout(screen.rect[2], screen.rect[3], n);
        if inside(screen.mouse, [x + w - 23., y + 3., 20., 18.]) {
            let v = s.config.get("gui_opacity") - 0.25;
            s.config.set("gui_opacity", if v < 0. { 1. } else { v });
            save(s);
            return;
        }
        for i in 0..=n {
            let r = row_rect(x, y, w, step, i);
            if !inside(screen.mouse, r) {
                continue;
            }
            s.selected = i;
            if i == n {
                if s.page == 0 {
                    s.close_requested = true;
                } else {
                    s.page = 0;
                }
                s.drag = None;
                return;
            }
            if s.page == 0 {
                s.page = i + 1;
                s.selected = 0;
                return;
            }
            if i == rs.len() {
                s.config
                    .reset(&rs.iter().map(|r| r.key).collect::<Vec<_>>());
                save(s);
                return;
            }
            let row = rs[i];
            if boolean(row) {
                s.config.set(row.key, 1. - s.config.get(row.key));
                save(s);
            } else if row.key == "modifier_key" {
                s.config.set(row.key, (s.config.get(row.key) + 1.) % 9.);
                save(s);
            } else {
                s.drag = Some(i);
                set_slider(
                    s,
                    row,
                    f64::from(horizon_client::ui::slider_value(
                        r[0],
                        r[2],
                        screen.mouse[0],
                    )),
                );
            }
            return;
        }
    });
}
pub(super) fn drag(game: &Game<'_>, screen: &NativeScreenView<'_>, canceled: &mut bool) {
    if !enabled(game) || game.native_mod_screen() != Some(ID) {
        return;
    }
    with(|s| {
        if let Some(i) = s.drag {
            if let Some(row) = rows(s.page).get(i).copied() {
                let (x, y, w, step) =
                    layout(screen.rect[2], screen.rect[3], rows(s.page).len() + 1);
                let r = row_rect(x, y, w, step, i);
                set_slider(
                    s,
                    row,
                    f64::from(horizon_client::ui::slider_value(
                        r[0],
                        r[2],
                        screen.mouse[0],
                    )),
                );
                *canceled = true;
            }
        }
    });
}
pub(super) fn release(game: &Game<'_>, canceled: &mut bool) {
    if enabled(game) && game.native_mod_screen() == Some(ID) {
        with(|s| s.drag = None);
        *canceled = true;
    }
}
pub(super) fn screen_key(game: &Game<'_>, key: &str, canceled: &mut bool) {
    if !enabled(game) || game.native_mod_screen() != Some(ID) {
        return;
    }
    with(|s| {
        s.keyboard_focus = true;
        let rs = rows(s.page);
        let n = if s.page == 0 {
            PAGES.len() - 1
        } else {
            rs.len() + 1
        };
        match key {
            "key.keyboard.escape" => {
                if s.page != 0 {
                    s.page = 0;
                    s.selected = 0;
                    *canceled = true;
                }
            }
            "key.keyboard.tab" | "key.keyboard.down" => {
                s.selected = (s.selected + 1) % (n + 1);
                *canceled = true;
            }
            "key.keyboard.up" => {
                s.selected = (s.selected + n) % (n + 1);
                *canceled = true;
            }
            "key.keyboard.left" | "key.keyboard.right" | "key.keyboard.home"
            | "key.keyboard.end" => {
                if let Some(row) = rs.get(s.selected).copied() {
                    let v = match key {
                        "key.keyboard.home" => row.min,
                        "key.keyboard.end" => row.max,
                        _ => {
                            s.config.get(row.key)
                                + if key == "key.keyboard.left" {
                                    -row.step
                                } else {
                                    row.step
                                }
                        }
                    };
                    s.config.set(row.key, v);
                    save(s);
                    *canceled = true;
                }
            }
            "key.keyboard.enter" | "key.keyboard.space" => {
                if s.selected == n {
                    if s.page != 0 {
                        s.page = 0;
                        s.selected = 0;
                    } else {
                        s.close_requested = true;
                    }
                    *canceled = true;
                } else if s.page == 0 {
                    s.page = s.selected + 1;
                    s.selected = 0;
                    *canceled = true;
                } else if s.selected == rs.len() {
                    s.config
                        .reset(&rs.iter().map(|r| r.key).collect::<Vec<_>>());
                    save(s);
                    *canceled = true;
                } else if let Some(row) = rs.get(s.selected).copied() {
                    if boolean(row) {
                        s.config.set(row.key, 1. - s.config.get(row.key));
                        save(s);
                    }
                    *canceled = true;
                }
            }
            _ => {}
        }
    });
}
pub(super) fn settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id != ID {
        return;
    }
    for label in PAGES {
        buttons.push(text(game, label).into());
    }
    let keys = KEYS
        .iter()
        .map(|(name, label, default)| [(*name).into(), text(game, label).into(), (*default).into()])
        .collect();
    if let Some(page) = game
        .settings
        .mods
        .installed
        .iter_mut()
        .find(|row| row.id == ID)
        .and_then(|row| row.native.as_mut())
    {
        page.keys = keys;
    }
}
pub(super) fn settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != ID || !enabled(game) || button >= PAGES.len() {
        return;
    }
    if game.player.is_some() && game.title.is_none() {
        game.screen = Screen::Playing;
        with(|s| {
            init(game, s);
            open(game, s, button);
        });
    }
}
pub(super) fn command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    let mut words = command.split_whitespace();
    if words.next() != Some("camerautils") || !enabled(game) {
        return;
    }
    *ran = true;
    let args: Vec<&str> = words.collect();
    with(|s| {
        init(game, s);
        match args.as_slice() {
            ["set", key, value] => {
                let parsed = if *key == "modifier_key" {
                    MODIFIERS
                        .iter()
                        .position(|m| m == value)
                        .map(|v| v as f64)
                        .or_else(|| value.parse().ok())
                } else {
                    match *value {
                        "true" => Some(1.),
                        "false" => Some(0.),
                        _ => value.parse().ok(),
                    }
                };
                if let Some(v) = parsed.filter(|v| v.is_finite()) {
                    if s.config.set(key, v) {
                        save(s);
                        game.native_chat(&format!("Camera Utils: {key} = {}", s.config.get(key)));
                        return;
                    }
                }
                game.native_chat(text(game, "Invalid setting or value."));
            }
            ["reload"] => {
                if let Some(path) = &s.path {
                    if let Ok(text) = std::fs::read_to_string(path) {
                        s.config = Config::parse(&text);
                    }
                }
            }
            ["reset"] => {
                s.config = Config::default();
                save(s);
            }
            ["get", key] => {
                if SPECS.iter().any(|sp| sp.key == *key) {
                    game.native_chat(&format!("{key} = {}", s.config.get(key)));
                }
            }
            ["settings"] | [] => {
                game.screen = Screen::Playing;
                open(game, s, 0);
            }
            _ => game.native_chat(
                "/camerautils settings | set <key> <value> | get <key> | reset | reload",
            ),
        }
    });
}
