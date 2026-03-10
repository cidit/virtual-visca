use bevy::{color::palettes::css::GRAY, input::keyboard::KeyboardInput, prelude::*};
use clap::{self, Parser};
use grafton_visca::{self, command::{const_encoding::constants::pan_tilt, EncodeVisca}, types::PanSpeed};
use std::{
    net::{SocketAddr, UdpSocket},
    str::FromStr,
};

use virtual_visca::DecodeVisca;

#[derive(Resource)]
struct UdpSocketResource(UdpSocket);

#[derive(Resource, Default)]
struct ViscaDriverConfig {
    expect_header: bool,
}

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

impl PTZCameraPlugin {
    fn sys_spawn_camera(mut commands: Commands) {
        commands.spawn((
            Camera3d::default(),
            MyCamera,
            Transform::from_xyz(0., 1.6, 3.).looking_at(Vec3::ZERO, Vec3::Y),
        ));
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
                        match direction {
                            // TODO: this isnt right. we should be setting a delta of change, and resetting that delta on the Stop event.
                            Up => cam_transform.rotate_local_x(1. * time.delta_secs()),
                            Down => cam_transform.rotate_local_x(-1. * time.delta_secs()),
                            Left => cam_transform.rotate_axis(Dir3::Y, 1. * time.delta_secs()),
                            Right => cam_transform.rotate_axis(Dir3::Y, -1. * time.delta_secs()),
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
                    }
                    other => {
                        println!("unimplemented command: {other:?}")
                    }
                },
            }
        }
    }

    fn sys_ptz_keyboard_controls(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut evt: EventWriter<ViscaCommand>,
        mut cam_transform: Single<&mut Transform, With<MyCamera>>,
        time: Res<Time>,
    ) {
        let are_pressed = |keycodes: &[KeyCode]| keycodes.iter().any(|&k| keyboard.pressed(k));

        if are_pressed(&[KeyCode::ArrowRight, KeyCode::KeyD]) {
            evt.write(ViscaCommand::PanTilt(
                grafton_visca::command::PanTilt::Move {
                    direction: grafton_visca::PanTiltDirection::Right,
                    pan_speed: grafton_visca::types::PanSpeed::MIN,
                    tilt_speed: grafton_visca::types::TiltSpeed::MIN,
                },
            ));
        }
        if are_pressed(&[KeyCode::ArrowLeft, KeyCode::KeyA]) {
            cam_transform.rotate_axis(Dir3::Y, 1. * time.delta_secs());
        }
        if are_pressed(&[KeyCode::ArrowUp, KeyCode::KeyW]) {
            cam_transform.rotate_local_x(1. * time.delta_secs());
        }
        if are_pressed(&[KeyCode::ArrowDown, KeyCode::KeyS]) {
            cam_transform.rotate_local_x(-1. * time.delta_secs());
        }
    }
}

impl Plugin for PTZCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::sys_spawn_camera)
            .add_systems(Update, Self::sys_ptz_keyboard_controls);
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
        .add_systems(Update, (sys_draw_gizmos, sys_esc_quits_game))
        .run();
}
