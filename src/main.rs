use bevy::{
    color::palettes::css::GRAY, input::keyboard::KeyboardInput, prelude::*,
};
use clap::{self, Parser};
use grafton_visca;
use std::{
    net::{SocketAddr, UdpSocket},
    str::FromStr,
};

// use virtual_visca::DecodeVisca;

#[derive(Resource)]
struct UdpSocketResource(UdpSocket);

#[derive(Resource, Default)]
struct ViscaDriverConfig {
    expect_header: bool,
}

/**
 * this guy's whole job is to read the network and emit the received Visca commands as events.
 */
struct ViscaDriverPlugin {
    socket: SocketAddr,
}

impl ViscaDriverPlugin {
    fn rcv_and_emit(
        // mut visca_events: EventWriter<ViscaCommand>,
        socket: ResMut<UdpSocketResource>,
        cfg: Res<ViscaDriverConfig>,
    ) {
        let mut buf = [0; 16];
        let (num, src) = match socket.0.recv_from(&mut buf) {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return, // no data was read
            Err(e) => panic!("encountered IO error: {e}"),
            Ok(ok) => ok,
        };

        if num == 0 {
            return; // no data
        }

        println!("recv {num} bytes from {src}: {buf:?}");

        if cfg.expect_header {
            println!("we're expecting headers, but dont do anything with it")
        }
    }
}

impl Plugin for ViscaDriverPlugin {
    fn build(&self, app: &mut App) {
        let socket = UdpSocket::bind(self.socket).unwrap();
        socket.set_nonblocking(true).unwrap();

        app.insert_resource(UdpSocketResource(socket))
            .add_systems(Update, Self::rcv_and_emit)
            .init_resource::<ViscaDriverConfig>();
    }
}

#[derive(Event)]
enum ViscaCommand {
    PanTilt(grafton_visca::command::PanTilt),
    Zoom(grafton_visca::command::zoom::Zoom),
}

struct PTZCameraPlugin;

#[derive(Component)]
struct MyCamera;

#[derive(Component, Default)]
struct PTZVelocity {
    pan: f32,
    tilt: f32,
    zoom: f32,
}

#[derive(Resource)]
struct CameraConfig {
    pan_tilt_speed: f32,
    zoom_speed: f32,
    max_zoom: f32,
    min_zoom: f32,
}

impl Default for CameraConfig {
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
 * this guy's whole job is to emulate cameras and respond to inputs (visca command events or KBd)
 * FIXME: doesnt need to treat kb inputs. should be handled in a system separate from this p^lugin.
 */
impl PTZCameraPlugin {
    fn sys_spawn_camera(mut commands: Commands) {
        commands.spawn((
            Camera3d::default(),
            MyCamera,
            Transform::from_xyz(0., 1.6, 3.).looking_at(Vec3::ZERO, Vec3::Y),
            PTZVelocity::default(),
        ));
    }

    fn sys_apply_camera_velocity() {
        todo!()
    }

    /// the big thing. applies the commands to the camera
    fn sys_interpret_visca_commands(
        mut cmds: EventReader<ViscaCommand>,
        mut cam_transform: Single<&mut Transform, With<MyCamera>>,
        time: Res<Time>,
    ) {
        for cmd in cmds.read() {
            use ViscaCommand::*;
            match cmd {
                PanTilt(pt) => match pt {
                    grafton_visca::command::PanTilt::Move {
                        direction,
                        pan_speed,
                        tilt_speed,
                    } => {
                        use grafton_visca::PanTiltDirection::*;
                        /// map a u8 to a value between 0.0 and 2.0
                        fn u8_to_scale_factor(val: u8) -> f32 {
                            2. * (val as f32) / ((2 ^ 8) as f32)
                        }
                        let pan_speed = u8_to_scale_factor(pan_speed.value()) * time.delta_secs();
                        let tilt_speed = u8_to_scale_factor(tilt_speed.value()) * time.delta_secs();
                        match direction {
                            // TODO: this isnt right. we should be setting a delta of change, and resetting that delta on the Stop event.
                            Up => cam_transform.rotate_local_x(tilt_speed),
                            Down => cam_transform.rotate_local_x(-tilt_speed),
                            Left => cam_transform.rotate_axis(Dir3::Y, pan_speed),
                            Right => cam_transform.rotate_axis(Dir3::Y, -pan_speed),
                            UpLeft => {
                                // TODO: extract a pan_tilt_by(&transform, pan: f32, tilt: f32) function?
                                cam_transform.rotate_local_x(1. * time.delta_secs());
                                cam_transform.rotate_axis(Dir3::Y, 1. * time.delta_secs());
                            }
                            UpRight => todo!(),
                            DownLeft => todo!(),
                            DownRight => todo!(),
                            Stop => todo!(),
                        }
                    },
                    other => {
                        println!("unimplemented command: {other:?}")
                    }
                },
                Zoom(zoom) => todo!(),
            }
        }
    }
}

impl Plugin for PTZCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::sys_spawn_camera)
            .add_systems(Update, Self::sys_interpret_visca_commands)
            .add_event::<ViscaCommand>();
    }
}

fn sys_esc_quits_game(mut exit: EventWriter<AppExit>, mut kb_events: EventReader<KeyboardInput>) {
    for event in kb_events.read() {
        if event.key_code == KeyCode::Escape {
            exit.write(AppExit::Success);
        };
    }
}

fn sys_draw_gizmos(mut gizmos: Gizmos, _time: Res<Time>) {
    gizmos.grid(
        Quat::from_rotation_x(std::f32::consts::PI / 2.),
        UVec2::splat(20),
        Vec2::new(2., 2.),
        GRAY,
    );
}

fn sys_ptz_keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut evt: EventWriter<ViscaCommand>,
) {
    let up = keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]);
    let down = keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]);
    let left = keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]);
    let right = keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]);

    let speed = if keyboard.any_pressed([KeyCode::ShiftRight, KeyCode::ShiftLeft]) {
        grafton_visca::types::SpeedLevel::Fast
    } else if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        grafton_visca::types::SpeedLevel::Slow
    } else {
        grafton_visca::types::SpeedLevel::Medium
    };

    let direction = match (up, down, left, right) {
        (true, false, true, false) => grafton_visca::PanTiltDirection::UpLeft,
        (true, false, false, true) => grafton_visca::PanTiltDirection::UpRight,
        (true, false, false, false) => grafton_visca::PanTiltDirection::Up,
        (false, true, true, false) => grafton_visca::PanTiltDirection::DownLeft,
        (false, true, false, true) => grafton_visca::PanTiltDirection::DownRight,
        (false, true, false, false) => grafton_visca::PanTiltDirection::Down,
        (false, false, true, false) => grafton_visca::PanTiltDirection::Left,
        (false, false, false, true) => grafton_visca::PanTiltDirection::Right,
        _ => grafton_visca::PanTiltDirection::Stop,
    };

    println!("{direction:?}");
    evt.write(ViscaCommand::PanTilt(
        grafton_visca::command::PanTilt::Move {
            direction,
            pan_speed: speed.into(),
            tilt_speed: speed.into(),
        },
    ));

    let zoom = match (up, down, left, right) {
        (true, true, false, false) => {
            grafton_visca::command::zoom::Zoom::TeleVariable(speed.into())
        }
        (false, false, true, true) => {
            grafton_visca::command::zoom::Zoom::WideVariable(speed.into())
        }
        _ => grafton_visca::command::zoom::Zoom::Stop,
    };

    println!("{zoom:?}");
    evt.write(ViscaCommand::Zoom(zoom));
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, value_name = "ADDRESS:PORT", default_value = "127.0.0.1:1259")]
    visca_bind: String,

    #[arg(long, value_name = "ADDRESS:PORT")]
    video_bind: Option<String>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn main() {
    let cli = Cli::parse();

    if cli.debug > 0 {
        println!("Hello, virtual-visca!");
    }

    let visca =
        SocketAddr::from_str(&cli.visca_bind).expect("could not parse `visca_bind` argument");
    if cli.debug > 0 {
        dbg!(visca);
    }

    App::new()
        .add_plugins((DefaultPlugins, PTZCameraPlugin))
        .add_plugins(ViscaDriverPlugin { socket: visca })
        .add_systems(
            Update,
            (
                sys_draw_gizmos,
                sys_esc_quits_game,
                sys_ptz_keyboard_controls.run_if(on_event::<KeyboardInput>),
            ),
        )
        .run();
}
