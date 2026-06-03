use std::{
  io, net::{
      IpAddr, Ipv4Addr,
      SocketAddr, TcpListener, Shutdown
  }
};
use rust_file_transfer::{
  DISCOVER_MSG, DISCOVER_RESPONSE_MSG,
  receive_message, send_message, receive_file
};



pub struct Server {
  ip: IpAddr,
  port: u16,
}

impl Server {
  pub fn new(port: u16) -> Self {
    Self {
      ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
      port,
    }
  }

  pub fn start(&self) -> io::Result<()> {
    let device = TcpListener::bind(SocketAddr::new(self.ip, self.port))?;
    println!("Waiting for client...");

    let mut entered = false;

    for stream in device.incoming() {
      let stream = stream?;
      if !entered {
        entered = true;
          
        let msg = receive_message(&stream)?;

        if msg == DISCOVER_MSG {
          send_message(&stream, DISCOVER_RESPONSE_MSG)?;
        } else {
          stream.shutdown(Shutdown::Both)?;
        }
        continue;
      }
      if let Ok(ip) = stream.peer_addr() {
        println!("Connected to {ip}");
      }

      receive_file(&stream)?;
      break;
    }
    Ok(())    
  }
}
