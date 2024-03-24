use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use crate::TerrainSlice;

#[derive(Component)]
pub struct MainCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for MainCamera {
    fn default() -> Self {
        MainCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn update_camera(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    input_keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut MainCamera, &mut Transform, &Projection)>,
    terrain_slice: Res<TerrainSlice>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = KeyCode::AltLeft;
    let pan_button = MouseButton::Middle;
    let window = windows.single();
    let zoom_mode = input_keys.pressed(KeyCode::ControlLeft);

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_keys.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta * 0.25;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.read() {
            pan += ev.delta * 0.75;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if input_keys.just_released(orbit_button) || input_keys.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(window);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(window);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let forward = transform.rotation * -Vec3::Z * pan.y;
            let flat = Vec3::new(forward.x, 0., forward.z);
            // make panning proportional to distance away from focus point
            let translation = (right + flat) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            if zoom_mode {
                any = true;
                pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
                // dont allow zoom to reach zero or you get stuck
                pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
            } else {
                any = true;
                pan_orbit.focus.y = terrain_slice.y as f32;
            }
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

fn get_primary_window_size(window: &Window) -> Vec2 {
    Vec2::new(window.width(), window.height())
}

/// Spawn a camera like this
pub fn setup_camera(mut cmd: Commands) {
    let translation = Vec3::new(0., 64., 0.);
    let radius = translation.length();
    let focus = Vec3::new(32., 50., 32.);

    cmd.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: std::f32::consts::PI / 7.0,
                ..Default::default()
            }),
            ..Default::default()
        },
        MainCamera {
            radius,
            focus,
            ..Default::default()
        },
    ));
}
