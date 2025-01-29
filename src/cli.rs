use crate::utils::{human2bytes, get_file_path_or_stdin, stat2bsize};

use getopts::Options;
use std::env;
use nix::sys::statvfs::statvfs;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub file_path: String,
    pub size: Option<u64>,
    pub rate_limit: Option<u64>,
    pub buffer_size: u64,
}

pub fn getopts() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optopt("s", "size", "set estimated data size to SIZE bytes", "SIZE");
    opts.optopt("L", "rate-limit", "limit transfer to RATE bytes per second", "RATE");
    opts.optopt("B", "buffer-size", "use a buffer size of BYTES", "BYTES");
    opts.optflag("h", "help", "show this help and exit");
    opts.optflag("V", "version", "show version information and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            eprintln!("Error: Failed to parse options");
            std::process::exit(1);
        }
    };

    Config {
        // TODO: This assignment is ugly...
        file_path: get_file_path_or_stdin(matches.free.get(0).map(|s| s.as_str())),

        size: match opts.parse(&args[1..]) {
            Ok(matches) => {
                if let Some(size) = matches.opt_str("s") {
                    // Use the value passed by user
                    Some(human2bytes(&size))
                } else if let Some(file_path) = matches.free.get(0) {
                    // Use the file size if it exists
                    let file_path = file_path.to_string();
                    if std::path::Path::new(&file_path).exists() {
                        Some(std::fs::metadata(&file_path).unwrap().len())
                    } else {
                        // File does not exist
                        None
                    }
                } else {
                    // No size or file path
                    None
                }
            }
            Err(_) => None,
        },

        rate_limit: match opts.parse(&args[1..]) {
            Ok(matches) => matches.opt_str("L").map(|l| human2bytes(&l)),
            Err(_) => None,
        },

        // TODO: This assingment is ugly...
        buffer_size: match statvfs(get_file_path_or_stdin(matches.free.get(0).map(|s| s.as_str())).as_str()) {
            Ok(stat) => {
                match match opts.parse(&args[1..]) {
                    // If a specific buffer size is passed by user, use it
                    Ok(matches) => matches.opt_str("B").map(|b| human2bytes(&b)),
                    Err(_) => None,
                } {
                    Some(buffer_size) => buffer_size,
                    None => {
                        // Use a different method to get the file system block size
                        let block_size = stat2bsize(&stat);
                        let buffer_size = (block_size as u64) * 32;
                        // If buffer_size is more than 512kb, set it to 512kb
                        std::cmp::min(buffer_size, 512 * 1024)
                    }
                }
            }
            Err(_) => {
                // If the block size cannot be determined, set buffer_size to 400kb
                400 * 1024
            }
        },

    }
}
