use bevy::prelude::*;
use itertools::Itertools;

use std::net::{SocketAddr, UdpSocket};

use crate::visca_impl;

#[derive(Resource)]
pub struct UdpSocketResource(pub UdpSocket);

#[derive(Resource, Default)]
pub struct ViscaDriverConfig {
    // expect_header: bool,
}


pub struct ViscaSocketPlugin {
    pub socket: SocketAddr,
}

fn receive_packet(mut msgs: MessageReader<visca_impl::Command>, socket: ResMut<UdpSocketResource>) {
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

/// Plugin that handles the socket interactions of Visca cameras
/// the control flow is as follows:
/// Socket -> decode -> commands -> events -> cameras
pub fn plugin(app: &mut App) {
    app.add_systems(Update, receive_packet)
        .init_resource::<ViscaDriverConfig>();
}
