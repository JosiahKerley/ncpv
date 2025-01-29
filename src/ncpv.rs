use crate::cli::Config;
use std::io::{self, Read, Write};

pub fn pv(config: Config) -> io::Result<()> {
    let start_time = std::time::Instant::now();
    let mut read_bytes = 0;
    let mut reader: Box<dyn Read> = match config.file_path.as_str() {
        "/dev/stdin" => Box::new(io::stdin()),
        _ => Box::new(std::fs::File::open(config.file_path)?),
    };
    let mut writer = io::stdout();
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        read_bytes += bytes_read as u64;
        writer.write_all(&buffer[..bytes_read])?;
        writer.flush()?;
    }
    Ok(())
}
