use std::{
  io::{self, Write},
  net::{
      IpAddr, Ipv4Addr, 
      SocketAddr, TcpStream
    },
    path::PathBuf, sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::Duration
};
use rust_file_transfer::{
  DISCOVER_MSG, DISCOVER_RESPONSE_MSG, DeviceIPS, get_file_metadata, get_input, receive_message, send_file, send_message
};


pub struct Client {
  local_ip: IpAddr,
  port: u16,
}

impl Client {
  pub fn new(port: u16) -> Self {
    let ips = DeviceIPS::device_ips();
    Self {
        local_ip: ips.local_ip,
        port,
    }
  }
  
  fn scan_subnet_parallel(base_ip: Ipv4Addr, port: u16) -> Result<Vec<IpAddr>, io::Error> {
    let octets = base_ip.octets();

    let mut handles: Vec<JoinHandle<Result<(), io::Error>>> = vec![];
    let detected_ips = Arc::new(Mutex::new(Vec::<IpAddr>::new()));

    for i in 1..=254 {
      let detected_ips_clone = Arc::clone(&detected_ips);
      let target_ip = Ipv4Addr::new(octets[0], octets[1], octets[2], i);
      let addr = SocketAddr::new(IpAddr::V4(target_ip), port);

      let handle = thread::spawn(move || {
        let timeout = Duration::from_millis(150);

        if let Ok(stream) = TcpStream::connect_timeout(&addr, timeout) {
          if let Ok(mut lock) = detected_ips_clone.lock() {
            send_message(&stream, DISCOVER_MSG)?;

            let message = receive_message(&stream)?;
            if message == DISCOVER_RESPONSE_MSG {
                lock.push(addr.ip());
            }
          }
            println!("Found device: {}", addr.ip());
        }

        Ok(())
      });

      handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    let a = detected_ips.lock().unwrap();
    Ok(a.clone())
  }


  fn make_connect(&self, ip_address: &IpAddr) -> io::Result<Option<TcpStream>> {
    let addr = SocketAddr::new(*ip_address, self.port);
    let client = TcpStream::connect(addr)?;

    println!("Connected to '{:?}'", client.peer_addr());

    Ok(
        Some(client)
    )
  }

  fn send(stream: &TcpStream, file_path: &PathBuf) -> io::Result<()> {
    let metadata = get_file_metadata(file_path)?;
    if metadata.is_file() {
        send_file(stream, file_path)?;
    } else {
        println!("Sending '{}' is NOT SUPPORTED. Enter a file name", file_path.display());
    }
    Ok(())
  }


  pub fn start(&self) -> io::Result<()> {
    if let Ok(ip) = self.local_ip.to_string().parse::<Ipv4Addr>(){
          
      loop {
        let list = Self::scan_subnet_parallel(ip, self.port)?;
        
        if list.is_empty() {
          println!("No Nearby Devices Found [ /r to retry ]");
          let response = get_input("> ");
          if response? == "/r" {
            continue;
          }
          return Ok(());
        }

        println!("Which Device would you like to connect to (/q to quit, /r to reload): ");
        for (index, addr) in list.iter().enumerate() {
          println!("[{index}] {addr}");
        }
          
        let mut response = String::new();
        loop {
          response.clear();
          print!("> ");
          io::stdout().flush()?;
          io::stdin().read_line(&mut response)?;
          response = response.trim().to_string();
            
          if response == "/q" { return Ok(()); }
          else if response == "/r" { break; }
          else {
            match response.parse::<u32>() {
              Ok(number) => {
                let index = number as usize;
                match list.get(index) {
                  None => {
                    println!("Your response should be within the range");
                    continue;
                  }

                  Some(ip) => {
                    if let Some(client) = self.make_connect(ip)? {
                      let mut file_path;
                      loop {
                        let input = get_input("Enter filename to send (/q to quit): ")?;
                        if input == "/q" { return Ok(()); }

                        file_path = PathBuf::from(input);
                        Self::send(&client, &file_path)?;
                      }

                    }
                  }

                }
              }
              
              Err(e) => {
                println!("Please ENTER a number: {e}");
                continue;
              }
            }
          }
        }
      }
    }

    Ok(())
  }    
}
