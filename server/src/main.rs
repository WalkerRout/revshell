
#[derive(Debug)]
struct Server {
  listener: std::net::TcpListener
}

type ResBErr<T> = Result<T, Box<dyn std::error::Error + 'static>>;

impl Server {
  fn new(con: &str) -> ResBErr<Self> {
    let listener = std::net::TcpListener::bind(con)?;
    Ok(Server { listener })
  }

  fn listen(&mut self, handle_connection_fn: impl Fn(std::net::TcpStream) -> ResBErr<()>) -> ResBErr<()> {

    for stream in self.listener.incoming() {
      match stream {
        Ok(stream) => {
          println!("New client from: {:?}", stream.peer_addr()?);
          handle_connection_fn(stream)?;
        },
        Err(e) => { 
          eprintln!("Error in accepting: {}", e); 
        }
      }
    }

    Ok(())
  }
}

fn handle_connection(mut stream: std::net::TcpStream) -> ResBErr<()> {
  use std::io::Read;
  use std::io::Write;

  loop {
    print!("> ");
      
    let mut cmd = String::new();

    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut cmd)?;
        
    stream.write(cmd.as_bytes())?;

    let mut buffer = vec![0; CAP];
    let mut reader = std::io::BufReader::new(&stream);
    reader.read(&mut buffer)?;

    let buffer_string = String::from_utf8_lossy(&buffer);

    println!("{}", buffer_string);
  }
  
  Ok(())
}

const CAP: usize = 12 * 1024;

fn main() -> ResBErr<()> {
  let ipp = "127.0.0.1:34254";
  let arg = std::env::args().nth(1);

  println!("Server Running on {} ...", ipp);

  let mut server = Server::new(ipp)?;
  server.listen(|stream| handle_connection(stream))?;
  
  Ok(())
}
