use bevy::{color::palettes::css::GRAY, input::keyboard::KeyboardInput, prelude::*};
use clap::{self, Parser};
use virtual_visca;

use std::{net::SocketAddr, str::FromStr};

fn sys_esc_quits_game(mut exit: MessageWriter<AppExit>, mut kb_events: MessageReader<KeyboardInput>) {
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

fn keyboard_changed(keyboard: Res<ButtonInput<KeyCode>>) -> bool {
    keyboard
        .get_just_pressed()
        .chain(keyboard.get_just_released())
        .count()
        != 0
}

/// FIXME: maybe should be separate for zoom and pantilt?
fn sys_ptz_keyboard_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut evt: MessageWriter<virtual_visca::visca::Command>,
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

    evt.write(virtual_visca::visca::Command::PanTilt(
        grafton_visca::command::PanTilt::Move {
            direction,
            pan_speed: speed.into(),
            tilt_speed: speed.into(),
        },
    ));

    let zoom = if keyboard.any_pressed([KeyCode::KeyQ, KeyCode::PageDown]) {
        grafton_visca::command::zoom::Zoom::WideVariable(speed.into())
    } else if keyboard.any_pressed([KeyCode::KeyE, KeyCode::PageUp]) {
        grafton_visca::command::zoom::Zoom::TeleVariable(speed.into())
    } else {
        grafton_visca::command::zoom::Zoom::Stop
    };

    evt.write(virtual_visca::visca::Command::Zoom(zoom));
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

    let visca_soket =
        SocketAddr::from_str(&cli.visca_bind).expect("could not parse `visca_bind` argument");
    if cli.debug > 0 {
        dbg!(visca_soket);
    }

    App::new()
        .add_plugins((
            DefaultPlugins,
            virtual_visca::ptz_camera::PTZCameraPlugin,
            virtual_visca::visca::ViscaDriverPlugin {
                socket: visca_soket,
            },
        ))
        .add_systems(
            Update,
            (
                sys_draw_gizmos,
                sys_esc_quits_game,
                sys_ptz_keyboard_controls.run_if(on_message::<KeyboardInput>.and(keyboard_changed)),
            ),
        )
        .run();
}
