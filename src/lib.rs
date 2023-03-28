use std::{io, net::UdpSocket, time::Duration, thread};
use regex::Regex;

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinecraftServer {
    pub title: String,
    pub addr: String
}

/// Scan the LAN network for minecraft servers. (takes ~1.5s)
pub fn scan_lan() -> io::Result<Vec<MinecraftServer>> {
    let socket = init_socket()?;
    let socket_port = socket.local_addr()?.port();

    let re_game_title = Regex::new(r"\[MOTD\](.*?)\[/MOTD\]").expect("regex failed");
    let re_game_port = Regex::new(r"\[AD\](.*?)\[/AD\]").expect("regex failed");

    let mut buf = [0; 1024];

    // Receive any LAN server info for 1.5 seconds:
    let receive_thread = thread::spawn(move || {
        let mut servers: Vec<MinecraftServer> = Vec::with_capacity(4);

        loop {
            let Ok((len, from)) = socket.recv_from(&mut buf) else {
                return servers;
            };

            if len == 1 { return servers; }

            let bytes = &buf[..len];
            let text = String::from_utf8_lossy(bytes);
    
            // Extract the game world title:
            let game_title = match re_game_title.captures(&text) {
                Some(caps) => match caps.get(1) {
                    Some(cap) => cap.as_str(),
                    None => "Unknown - Unknown World"
                },
                None => "Unknown - Unknown World"
            };
    
            // Extract the game world port:
            let game_port = match re_game_port.captures(&text) {
                Some(caps) => match caps.get(1) {
                    Some(cap) => cap.as_str(),
                    None => continue
                },
                None => continue
            };
    
            // Combine the port and address.
            let addr = format!("{}:{}", from.ip(), game_port);

            // Check if the server isn't already in the vector:
            if servers.iter().any(|s| s.addr == addr) == false {
                servers.push(MinecraftServer {
                    title: game_title.to_string(),
                    addr
               });
            }
        }
    });

    // Join the thread after 1500 milliseconds (1.5s)
    thread::sleep(Duration::from_millis(1500));
    let socket = UdpSocket::bind("127.0.0.1:26456")?;
    socket.send_to(&[0; 1], format!("127.0.0.1:{}", socket_port))?;
    
    Ok(receive_thread.join().expect("failed to join thread"))
}