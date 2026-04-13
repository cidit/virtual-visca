use bevy::prelude::*;
use grafton_visca::types::ZoomSpeed;
use grafton_visca::types::SpeedLevel;
use grafton_visca::types::PanSpeed;
use grafton_visca::types::TiltSpeed;

use crate::visca;

#[derive(Component, Default)]
pub struct PTZVelocity {
    pub pan: f32,
    pub tilt: f32,
    pub zoom: f32,
}

#[derive(Component)]
pub struct CameraSettings {
    pub pan_tilt_speed: f32,
    /// magnitude per seconds
    pub zoom_speed: f32,
    pub max_zoom: f32,
    pub min_zoom: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            pan_tilt_speed: 1.0,
            zoom_speed: 1.0,
            max_zoom: 5.0,
            min_zoom: 0.5,
        }
    }
}

/**
 * this guy's whole job is to emulate cameras and respond to received visca commands
 */
pub struct PTZCameraPlugin;

impl PTZCameraPlugin {
    fn sys_spawn_camera(mut commands: Commands) {
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0., 1.6, 3.).looking_at(Vec3::ZERO, Vec3::Y),
            PTZVelocity::default(),
            CameraSettings::default(),
        ));
    }

    fn sys_apply_camera_velocity(
        query: Query<(
            &mut Transform,
            &mut Projection,
            &PTZVelocity,
            &CameraSettings,
        )>,
        time: Res<Time>,
    ) {
        for (mut transform, projection, velocity, settings) in query {
            let Projection::Perspective(perspective) = projection.into_inner() else {
                continue;
            };
            transform.rotate_local_x(velocity.tilt * settings.pan_tilt_speed * time.delta_secs());
            transform.rotate_axis(
                Dir3::Y,
                velocity.pan * settings.pan_tilt_speed * time.delta_secs(),
            );
            let current_zoom = perspective.fov.tan().recip();
            let new_zoom = (current_zoom + velocity.zoom * settings.zoom_speed * time.delta_secs())
                .clamp(settings.min_zoom, settings.max_zoom);
            perspective.fov = new_zoom.recip().atan();
        }
    }

    /// the big thing. applies the commands to the camera
    fn sys_interpret_visca_commands(
        mut cmds: MessageReader<visca::Command>,
        mut query: Query<(&mut PTZVelocity, &mut Transform, &mut CameraSettings)>,
        time: Res<Time>,
    ) {
        pub(crate) fn scaled_u8(v: u8, min: u8, max: u8) -> f32 {
            // scales value up to a max of 2x unit
            2. * (v - min) as f32 / (max - min) as f32
        }
        for cmd in cmds.read() {
            for (mut velocity, transform, _settings) in query.iter_mut() {
                match cmd {
                    visca::Command::PanTilt(pan_tilt) => match pan_tilt {
                        grafton_visca::command::PanTilt::Home => todo!(),
                        grafton_visca::command::PanTilt::Reset => todo!(),
                        grafton_visca::command::PanTilt::Move {
                            direction,
                            pan_speed,
                            tilt_speed,
                        } => {
                            let tilt = scaled_u8(
                                tilt_speed.value(),
                                TiltSpeed::MIN.value(),
                                TiltSpeed::MAX.value(),
                            );
                            let pan = scaled_u8(
                                pan_speed.value(),
                                PanSpeed::MIN.value(),
                                PanSpeed::MAX.value(),
                            );

                            velocity.tilt = match direction {
                                grafton_visca::PanTiltDirection::Up
                                | grafton_visca::PanTiltDirection::UpLeft
                                | grafton_visca::PanTiltDirection::UpRight => tilt,
                                grafton_visca::PanTiltDirection::Down
                                | grafton_visca::PanTiltDirection::DownLeft
                                | grafton_visca::PanTiltDirection::DownRight => -tilt,
                                _ => 0.,
                            };
                            velocity.pan = match direction {
                                grafton_visca::PanTiltDirection::Left
                                | grafton_visca::PanTiltDirection::UpLeft
                                | grafton_visca::PanTiltDirection::DownLeft => pan,
                                grafton_visca::PanTiltDirection::Right
                                | grafton_visca::PanTiltDirection::UpRight
                                | grafton_visca::PanTiltDirection::DownRight => -pan,
                                _ => 0.,
                            }
                        }
                        other => println!("unimplemented PanTilt command: {other:?}"),
                    },
                    visca::Command::Zoom(zoom) => {
                        velocity.zoom = match zoom {
                            grafton_visca::command::zoom::Zoom::Stop => 0.,
                            grafton_visca::command::zoom::Zoom::TeleStd => scaled_u8(
                                SpeedLevel::Medium.to_zoom_speed(),
                                ZoomSpeed::MIN.value(),
                                ZoomSpeed::MAX.value(),
                            ),
                            grafton_visca::command::zoom::Zoom::WideStd => -scaled_u8(
                                SpeedLevel::Medium.to_zoom_speed(),
                                ZoomSpeed::MIN.value(),
                                ZoomSpeed::MAX.value(),
                            ),
                            grafton_visca::command::zoom::Zoom::TeleVariable(zoom_speed) => {
                                scaled_u8(zoom_speed.value(), ZoomSpeed::MIN.value(), ZoomSpeed::MAX.value())
                            }
                            grafton_visca::command::zoom::Zoom::WideVariable(zoom_speed) => {
                                -scaled_u8(zoom_speed.value(), ZoomSpeed::MIN.value(), ZoomSpeed::MAX.value())
                            }
                            other => {
                                println!("unimplemented Zoom command: {other:?}");
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Plugin for PTZCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::sys_spawn_camera)
            .add_systems(Update, Self::sys_interpret_visca_commands)
            .add_systems(Update, Self::sys_apply_camera_velocity);
    }
}
