/// Just some old testing code with debug statements...

use std::{io, net::UdpSocket};
use lazy_static::lazy_static;

/* -------- Multicast Socket -------- */
use socket2::{Socket, Domain, Type};
use async_net::{SocketAddr, Ipv4Addr};

const MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(224, 0, 2, 60);
const PORT: u16 = 4445;

/// Initialize a Minecraft LAN multicast listener.
fn init_socket() -> io::Result<UdpSocket> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), None)?;
    socket.set_reuse_address(true)?;
    #[cfg(target_vendor = "apple")]
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT)).into())?;
    socket.join_multicast_v4(&MULTICAST_GROUP, &Ipv4Addr::UNSPECIFIED)?;
    Ok(socket.into())
}

use regex::Regex;
lazy_static! {
    static ref GAME_TITLE: Regex = Regex::new(r"\[MOTD\](.*?)\[/MOTD\]").unwrap();
    static ref GAME_PORT: Regex = Regex::new(r"\[AD\](.*?)\[/AD\]").unwrap();
}

fn main() {
    let socket = init_socket().expect("failed to init multicast socket");

    let mut buf = [0; 1024];

    loop {
        let (len, from) = socket.recv_from(&mut buf).unwrap();
        let bytes = &buf[..len];
        let text = String::from_utf8_lossy(bytes);

        // Extract the game world title:
        let game_title = match GAME_TITLE.captures(&text) {
            Some(caps) => match caps.get(1) {
                Some(cap) => cap.as_str(),
                None => ""
            },
            None => ""
        };

        // Extract the game world port:
        let game_port = match GAME_PORT.captures(&text) {
            Some(caps) => match caps.get(1) {
                Some(cap) => cap.as_str(),
                None => panic!("World port missing!")
            },
            None => panic!("World port missing!")
        };

        // Combine the port and address.
        let addr = format!("{}:{}", from.ip(), game_port);

        println!("\nFound world: {}\nIP: {}", game_title, addr);
    }
}