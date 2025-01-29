use std::path::Path;

pub fn human2bytes(human: &str) -> u64 {
    let mut num = String::new();
    let mut unit = String::new();
    for c in human.chars() {
        if c.is_digit(10) {
            num.push(c);
        } else {
            unit.push(c);
        }
    }
    let num = match num.parse::<u64>() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: Could not parse the human number. Expected something like 10M, or 12345, but got: {}", human);
            std::process::exit(1);
        }
    };
    match unit.to_lowercase().as_str() {
        "k" => num * 1024,
        "m" => num * 1024 * 1024,
        "g" => num * 1024 * 1024 * 1024,
        "t" => num * 1024 * 1024 * 1024 * 1024,
        _ => num,
    }
}


fn bytes2human_unit(bytes: u64) -> (u64, String) {
    if bytes < 1024 {
        return (bytes, "B".to_string());
    }
    let kb = bytes as f64 / 1024.0;
    if kb < 1024.0 {
        return (kb as u64, "KB".to_string());
    }
    let mb = kb / 1024.0;
    if mb < 1024.0 {
        return (mb as u64, "MB".to_string());
    }
    let gb = mb / 1024.0;
    if gb < 1024.0 {
        return (gb as u64, "GB".to_string());
    }
    let tb = gb / 1024.0;
    (tb as u64, "TB".to_string())
}

pub fn bytes2human(bytes: u64) -> String {
    format!("{:.2} {}", bytes2human_unit(bytes).0 as f64, bytes2human_unit(bytes).1)
}

pub fn bytes2human_scale(bytes: u64) -> String {
    bytes2human(bytes).split_whitespace().last().unwrap().to_string()
}

pub fn seconds2human(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{}s", seconds);
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{}m", minutes);
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{}h", hours);
    }
    let days = hours / 24;
    if days < 7 {
        return format!("{}d", days);
    }
    let weeks = days / 7;
    if weeks < 4 {
        return format!("{}w", weeks);
    }
    let months = weeks / 4;
    if months < 12 {
        return format!("{}M", months);
    }
    let years = months / 12;
    format!("{}y", years)
}



pub fn get_file_path_or_stdin(canidate: Option<&str>) -> String {
    match canidate {
        Some(file_path) => {
            if Path::new(file_path).exists() {
                return file_path.to_string();
            } else {
                eprintln!("Error: File {} does not exist", file_path);
                std::process::exit(1);
            }
        }
        None => {
            return "/dev/stdin".to_string();
        }
    }
}

pub fn stat2bsize(stat: &nix::sys::statvfs::Statvfs) -> u64 {
    stat.block_size() as u64
}
