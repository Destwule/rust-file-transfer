use std::{
  fs::{self, File}, 
  io::{self, Read, Write}, 
  net::{
      TcpStream, IpAddr, UdpSocket
  }, 
  path::PathBuf
};

pub struct FileMetaData {
  pub file_name: String,
  pub file_size: usize,
}


const FILE_BUFFER_SIZE: usize = 32 * 1024;
pub const DISCOVER_MSG: &str = "DISCOVER";
pub const DISCOVER_RESPONSE_MSG: &str = "RUST TCP FILE TRANSFER";

pub fn get_input(prompt: &str) -> io::Result<String> {
  let mut response = String::new();
  print!("{prompt} ");
  io::stdout().flush()?;
  io::stdin().read_line(&mut response)?;
  response = response.trim().to_string();

  Ok(response)
}

pub fn send_message(mut stream: &TcpStream, message: &str) -> io::Result<()> {
  stream.write_all(&(message.len() as u64).to_be_bytes())?;
  stream.write_all(&message.as_bytes())?;
  Ok(())
}

pub fn receive_message(mut stream: &TcpStream) -> Result<String, io::Error> {
  let mut length_buffer = [0u8; 8];
  stream.read_exact(&mut length_buffer)?;

  let mut message_buffer = vec![0u8; u64::from_be_bytes(length_buffer) as usize];
  stream.read_exact(&mut message_buffer)?;
  
  Ok(
      String::from_utf8_lossy(&message_buffer).to_string()
  )
}

pub fn get_file_metadata(path: &PathBuf) -> Result<fs::Metadata, io::Error> {
  let metadata = match path.metadata() {
    Err(e) => {
      return Err(io::Error::new(e.kind(), "Could not get file metadata"));
    }
    Ok(metadata) => metadata
  };

  Ok(metadata)
}

pub fn send_metadata(mut stream: &TcpStream, path: &PathBuf) -> Result<FileMetaData, io::Error> {
  let metadata = get_file_metadata(path)?;
  let file_name = path.file_name().ok_or(
    io::Error::new(io::ErrorKind::InvalidFilename,
    "Could not get the filename")
  )?.to_string_lossy().to_string();

  let file_name_length = file_name.len();
  let file_size = metadata.len();

  let mut buffer = Vec::new();
  buffer.extend_from_slice(&(file_name_length as u64).to_be_bytes());
  buffer.extend_from_slice(&(file_name.as_bytes()));
  buffer.extend_from_slice(&(file_size.to_be_bytes()));

  stream.write_all(&mut buffer)?;

  Ok(FileMetaData {
    file_name,
    file_size: file_size as usize,
  })
}

pub fn send_file(mut stream: &TcpStream, path: &PathBuf) -> io::Result<()> {
  let metadata = send_metadata(&stream, &path)?;

  println!("Sending file... {}", metadata.file_name);

  let mut file = File::open(&path)?;
  let mut file_buffer = [0u8; FILE_BUFFER_SIZE];

  loop {
    let n = file.read(&mut file_buffer)?;
    
    if n == 0 {
        break;
    }

    stream.write_all(&file_buffer[..n])?;
  }
  
  println!("Done Sending: {}", metadata.file_name);

  Ok(())
}



pub fn receive_metadata(mut stream: &TcpStream) -> Result<FileMetaData, io::Error> {
  let mut file_name_length_buffer = [0u8; 8];
  stream.read_exact(&mut file_name_length_buffer)?;

  let mut file_name_buffer = vec![0u8; u64::from_be_bytes(file_name_length_buffer) as usize];
  stream.read_exact(&mut file_name_buffer)?;
  let file_name = String::from_utf8_lossy(&file_name_buffer).to_string();

  let mut file_size_len_buffer = [0u8; 8];
  stream.read_exact(&mut file_size_len_buffer)?;
  let file_size = u64::from_be_bytes(file_size_len_buffer) as usize;
  
  Ok(
    FileMetaData { file_name, file_size }
  )
}

pub fn receive_file(mut stream: &TcpStream) -> Result<FileMetaData, io::Error> {
  let metadata = receive_metadata(&stream)?;
  let file_size = metadata.file_size as u64;
  
  let mut received = 0u64;
  let mut file_buffer = [0u8; FILE_BUFFER_SIZE];

  let mut file = File::create(&metadata.file_name)?;

  println!("Receiving file... {}", metadata.file_name);
  
  while received < file_size {
    let buffer_to_receive = std::cmp::min(file_buffer.len() as u64, file_size - received);

    let n = stream.read(&mut file_buffer[..buffer_to_receive as usize])?;

    if n == 0 {
      return Err(io::Error::new(
        io::ErrorKind::ConnectionAborted,
        "The connection was aborted")
      );
    }

    received += n as u64;
    file.write_all(&file_buffer[..n])?;
  }

  println!("File Received Successfully... {}", metadata.file_name);

  Ok(metadata)
}



pub struct DeviceIPS {
  pub local_ip: IpAddr,
  pub broadcast_ip: IpAddr,
}

impl DeviceIPS {
  pub fn device_ips() -> Self {
    Self {
      local_ip: Self::local_ip(),
      broadcast_ip: Self::broadcast_ip(),
    }
  }

  fn local_ip() -> IpAddr {
    let local_ip = match Self::get_local_ip() {
      Ok(ip_addr) => ip_addr,
      Err(e) => {
        eprintln!("Error: {e}");
        std::process::exit(1);
      }
    };

    local_ip
  }

  fn broadcast_ip() -> IpAddr {
    let ip = match Self::get_broadcast_ip(&Self::local_ip()) {
      Ok(ip_addr) => ip_addr,
      Err(e) => {
        eprintln!("Error: {e}");
        std::process::exit(1);
      }
    };

    ip
  }
  
  fn get_local_ip() -> Result<IpAddr, io::Error> {
      let device = UdpSocket::bind("0.0.0.0:0")?;
      device.connect("1.2.3.4:80")?;
      let device_ip = device.local_addr()?.ip();

      Ok(device_ip)
  }

  fn get_broadcast_ip(ip_address: &IpAddr) -> Result<IpAddr, io::Error> {
    match ip_address {
      IpAddr::V4(ipv4) => {
        use std::str::FromStr;
    
        let ip = ipv4.to_string();
        let mut ip_list = ip.split(".").collect::<Vec<&str>>();
        ip_list[3] = "255";
        let c= ip_list.join(".");
    
        IpAddr::from_str(&c).map_err(|e|
          io::Error::new(io::ErrorKind::InvalidInput, e)
        )
      }
      IpAddr::V6(_) => Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "IPV6 not supported for broadcast"
      ))
    }
  }

}