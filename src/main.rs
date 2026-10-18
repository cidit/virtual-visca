use bevy::{color::palettes::css::GRAY, input::keyboard::KeyboardInput, prelude::*};
use bevy::feathers::{theme::{*}, dark_theme::create_dark_theme};
use bevy_inspector_egui as inspector;
use clap::{self, Parser};
use virtual_visca::{self, visca_camera::ViscaEnabled};

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

fn sys_esc_quits_game(
    mut exit: MessageWriter<AppExit>,
    mut kb_events: MessageReader<KeyboardInput>,
) {
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



fn spawn_visca_camera(mut commands: Commands) {
    let translation = Vec3::new(0., 1.6, 3.);
    commands.spawn_scene(bsn! {
        ViscaEnabled
        Transform {
            translation: translation,
            // rotation: Quat::look_to_lh(-translation, Vec3::Y),
        }
    });
}


fn main() {
    let cli = virtual_visca::cli::Cli::parse();

    if cli.debug > 0 {
        println!("Hello, virtual-visca!");
    }

    let visca_socket =
        SocketAddr::from_str(&cli.visca_bind).expect("could not parse `visca_bind` argument");
    if cli.debug > 0 {
        dbg!(visca_socket);
    }

    let socket = UdpSocket::bind(visca_socket).unwrap();
    socket.set_nonblocking(true).unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins,
            // FeathersPlugins,
            inspector::bevy_egui::EguiPlugin::default(),
            inspector::quick::WorldInspectorPlugin::new(),
            virtual_visca::visca_camera::plugin,
            // virtual_visca::visca_socket::plugin,
            virtual_visca::visca_controls::plugin,
        ))
        // .insert_resource(virtual_visca::visca_socket::UdpSocketResource(socket))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_message::<virtual_visca::visca_impl::Command>()
        .add_systems(Startup, spawn_visca_camera)
        .add_systems(
            Update,
            (
                sys_draw_gizmos,
                sys_esc_quits_game,
            ),
        )
        .run();
}
