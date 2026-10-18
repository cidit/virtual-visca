use bevy::{
    feathers::{
        constants::fonts, controls::FeathersButton, font_styles::InheritableFont, theme::ThemedText,
    },
    input::keyboard::KeyboardInput,
    prelude::*,
    ui_widgets::{Activate, ActivateOnPress},
};

use crate::visca_impl;

#[derive(Component, Clone, Default)]
struct ViscaCommandTrigger(Option<visca_impl::Command>);

impl ViscaCommandTrigger {
    pub fn from(cmd: visca_impl::Command) -> Self {
        Self(Some(cmd))
    }
}

fn visca_btn(label: &'static str, cmd: visca_impl::Command) -> impl Scene {
    bsn! {
        Name(label)
        Button
        @FeathersButton {
            @caption: bsn! {Text(label) ThemedText}
        }
        Node {}
        BackgroundColor(Color::BLACK)
        ViscaCommandTrigger::from(cmd)
        ActivateOnPress
        on(move |_activate: On<Activate>| {
            info!("{label}!");
        })
    }
}

fn menu_scene() -> impl Scene {
    use grafton_visca::command::PanTilt;
    use grafton_visca::{PanTiltDirection, SpeedLevel};

    let base_move_cmd = PanTilt::Move {
        direction: PanTiltDirection::Up,
        pan_speed: SpeedLevel::Medium.into(),
        tilt_speed: SpeedLevel::Medium.into(),
    };
    let ptz_buttons = [
        ("up left", PanTiltDirection::UpLeft),
        ("up", PanTiltDirection::Up),
        ("up right", PanTiltDirection::UpRight),
        ("left", PanTiltDirection::Left),
        ("stop", PanTiltDirection::Stop),
        ("right", PanTiltDirection::Right),
        ("down left", PanTiltDirection::DownLeft),
        ("down", PanTiltDirection::Down),
        ("down right", PanTiltDirection::DownRight),
    ]
    .map(|(name, dir)| {
        (
            name,
            grafton_visca::command::PanTilt::Move {
                direction: dir,
                pan_speed: grafton_visca::SpeedLevel::Medium.into(),
                tilt_speed: grafton_visca::SpeedLevel::Medium.into(),
            },
        )
    })
    .map(|(name, grafton_cmd)| (name, ViscaCommandTrigger::from(grafton_cmd.into())))
    .map(|(name, visca_cmd_trigger)| {
        bsn! {
            Name(name)
           Button
           @FeathersButton {
               @caption: bsn! {Text("Up") ThemedText}
           }
           Node {
           //     width: percent(20),
           //     height: percent(20),
           }
           BackgroundColor(Color::BLACK)
           // visca_cmd_trigger
           ActivateOnPress
           on(|_activate: On<Activate>| {
               info!("UP");
           })
        }
    });

    bsn! {
       Node {
           width: percent(100),
           height: percent(100),
           justify_content: JustifyContent::End,
       }
       BackgroundColor(Color::NONE)
       InheritableFont { font: fonts::MONO }
       Children[
       ]
    }
}

fn gui_btns_to_commands(
    buttons: Query<(&Name, &Interaction, &ViscaCommandTrigger), Changed<Interaction>>,
    mut msg: MessageWriter<visca_impl::Command>,
) {
    println!("in sys");
    for (name, interaction, cmd) in buttons {
        println!("Button {name} got clicked");

        let ViscaCommandTrigger(Some(cmd)) = cmd else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                msg.write(cmd.clone());
            }
            _ => {}
        }
    }
}

/// FIXME: maybe should be separate for zoom and pantilt?
fn ptz_keyboard_to_commands(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut msg: MessageWriter<visca_impl::Command>,
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

    msg.write(visca_impl::Command::PanTilt(
        grafton_visca::command::PanTilt::Move {
            direction,
            pan_speed: speed.into(),
            tilt_speed: speed.into(),
        },
    ));

    let zoom = if keyboard.any_pressed([KeyCode::KeyQ, KeyCode::PageDown]) {
        grafton_visca::command::Zoom::WideVariable(speed.into())
    } else if keyboard.any_pressed([KeyCode::KeyE, KeyCode::PageUp]) {
        grafton_visca::command::Zoom::TeleVariable(speed.into())
    } else {
        grafton_visca::command::Zoom::Stop
    };

    msg.write(visca_impl::Command::Zoom(zoom));
}

fn keyboard_changed(keyboard: Res<ButtonInput<KeyCode>>) -> bool {
    keyboard
        .get_just_pressed()
        .chain(keyboard.get_just_released())
        .count()
        != 0
}

#[rustfmt::skip]
pub fn plugin(app: &mut App) {
    app.add_systems(Startup, menu_scene.spawn())
        .add_systems(Update, (
            ptz_keyboard_to_commands.run_if(on_message::<KeyboardInput>.and_then(keyboard_changed)),
            gui_btns_to_commands,
        ));
}
