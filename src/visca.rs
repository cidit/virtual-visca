use bevy::{prelude::*, time::common_conditions::paused};
use grafton_visca;
use itertools::Itertools;

use std::net::{SocketAddr, UdpSocket};

#[derive(Resource)]
pub struct UdpSocketResource(UdpSocket);

#[derive(Resource, Default)]
pub struct ViscaDriverConfig {
    // expect_header: bool,
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
    fn receive_packet(mut messages: MessageReader<Command>, socket: ResMut<UdpSocketResource>) {
        let mut buf = [0; 24];
        let (num, src) = match socket.0.recv_from(&mut buf) {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return, // no data was read
            Err(e) => panic!("encountered IO error: {e}"),
            Ok(ok) => ok,
        };

        println!("recv {num} bytes from {src}: {buf:?}");

        let payload = buf
            .into_iter()
            .skip(8) // skip UDP header
            .take_while(|&it| it != 0xFF) // take until terminator
            .skip(1) // skip camera address header
            .collect_vec();

        if payload.len() == 0 {
            return; // not a valid visca command packet
        }

        let payload = payload.into_iter();

        // if Some()
        
    }
}

impl Plugin for ViscaDriverPlugin {
    fn build(&self, app: &mut App) {
        let socket = UdpSocket::bind(self.socket).unwrap();
        socket.set_nonblocking(true).unwrap();

        app.insert_resource(UdpSocketResource(socket))
            .add_systems(Update, Self::receive_packet)
            .init_resource::<ViscaDriverConfig>()
            .add_message::<crate::visca::Command>();
    }
}
