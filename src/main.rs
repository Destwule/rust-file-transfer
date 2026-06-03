mod client;
mod server;

use std::io::{self, Write};


fn main() {
  let mut response = String::new();
  loop {
    println!();
    response.clear();
    println!("What would u like to do (/q to quit): ");
    println!("[1] Send File\n[2] Receive File");
    print!("> ");
    io::stdout()
      .flush()
      .expect("Could not flush to standard output");
    io::stdin()
      .read_line(&mut response)
      .expect("Could not read input");
    response = response.trim().to_string();

    match response.as_str() {
      "1" => {
        let client = client::Client::new(8080);
        if let Err(e) = client.start() {
          eprintln!("An Error Occured: {e}");
          break;
        }
      }
      "2" => {
        let server = server::Server::new(8080);
        if let Err(e) = server.start() {
          eprintln!("An Error Occured: {}", e.kind());
          break;
        }
      }
      "/q" => {
        break;
      }
      _ => {
        println!("Choose one of the options");
        continue;
      }
    }
  } 
}
