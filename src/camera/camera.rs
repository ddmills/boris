use bevy::{
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

pub struct CameraPlugin;

#[derive(Component)]
pub struct FlyCamera;

#[derive(Resource, Default)]
struct CameraState {
    reader_motion: ManualEventReader<MouseMotion>,
}

#[derive(Resource)]
struct CameraSettings {
    sensitivity: f32,
    speed: f32,
    shift_multiplier: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 20.,
            shift_multiplier: 2.,
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .init_resource::<CameraSettings>()
            .add_systems(Startup, initial_grab_cursor)
            .add_systems(Update, apply_camera_translation)
            .add_systems(Update, apply_camera_rotation)
            .add_systems(Update, grab_cursor);
    }
}

fn grab_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window)
        }
    } else {
        warn!("Primary window not found");
    }
}

fn apply_camera_rotation(
    settings: Res<CameraSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<CameraState>,
    motion: Res<Events<MouseMotion>>,
    mut cameras: Query<&mut Transform, With<FlyCamera>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in cameras.iter_mut() {
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            for ev in state.reader_motion.read(&motion) {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }
            }

            pitch = pitch.clamp(-1.54, 1.54);

            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    } else {
        warn!("Primary window not found");
    }
}

fn apply_camera_translation(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<CameraSettings>,
    mut cameras: Query<&mut Transform, With<FlyCamera>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in cameras.iter_mut() {
            let mut delta = Vec3::ZERO;
            let local_z = *transform.local_z();
            let forward = *transform.forward();
            let mut is_shift: bool = false;
            // let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => match key {
                        KeyCode::KeyW => delta += forward,
                        KeyCode::KeyS => delta -= forward,
                        KeyCode::KeyA => delta -= right,
                        KeyCode::KeyD => delta += right,
                        KeyCode::ShiftLeft => is_shift = true,
                        _ => (),
                    },
                }
            }

            delta = delta.normalize_or_zero();

            let speed = if is_shift {
                settings.speed * settings.shift_multiplier
            } else {
                settings.speed
            };

            transform.translation += delta * time.delta_seconds() * speed;
        }
    } else {
        warn!("Primary window not found");
    }
}

fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found");
    }
}
