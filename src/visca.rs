use bevy::prelude::*;
use grafton_visca;

use std::net::{SocketAddr, UdpSocket};

#[derive(Resource)]
pub struct UdpSocketResource(UdpSocket);

#[derive(Resource, Default)]
pub struct ViscaDriverConfig {
     expect_header: bool,
}

#[derive(Message)]
pub enum Command {
    PanTilt(grafton_visca::command::PanTilt),
    Zoom(grafton_visca::command::zoom::Zoom),
}

/**
 * this guy's whole job is to read the network and emit the received Visca commands as events.
 */
pub struct ViscaDriverPlugin {
    pub socket: SocketAddr,
}

impl ViscaDriverPlugin {
    fn rcv_and_emit(
        mut messages: MessageReader<Command>,
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
            .init_resource::<ViscaDriverConfig>()
            .add_message::<crate::visca::Command>();
    }
}

