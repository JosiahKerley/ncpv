use chrono::{DateTime, Local};
use std::time::{Duration, SystemTime};
use std::fmt::{Debug};
use crate::tui;
use crate::cli::Config;
use crate::utils::{bytes2human, seconds2human, bytes2human_scale};

use std::io::{self, Read, Write};

use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

const ETA_DELAY_SECONDS: u64 = 5;

#[derive(Debug)]
pub struct NCPV {
    config: Config,
    percent_complete: Option<f64>,
    stopwatch: std::time::Instant,
    start_time: SystemTime,
    eta: Option<std::time::Duration>,
    samples: Vec<u64>,
    read_bytes: u64,
    exit: bool,
}

impl NCPV {
    fn get_transfer_rate(&self) -> u64 {
        match self.stopwatch.elapsed().as_secs() {
            0 => 0,
            secs => self.read_bytes / secs,
        }
    }

    fn get_eta_text(&self, eta: Duration) -> String {
        match self.eta {
            Some(_) => match self.percent_complete {
                Some(percent) => {
                    if percent == 100.0 {
                        "ETA: Done".to_string()
                    } else if percent >= 50.0 {
                        format!("ETA: {}", seconds2human(eta.as_secs())).to_string()
                    } else if percent >= 25.0 {
                        format!("ETA: ~{}", seconds2human(eta.as_secs())).to_string()
                    } else {
                        format!("ETA: {}?", seconds2human(eta.as_secs())).to_string()
                    }
                }
                None => "ETA: N/A".to_string(),
            },
            None => "ETA: N/A".to_string(),
        }
    }


    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    pub fn run(&mut self, terminal: &mut tui::Tui, config: Config) -> io::Result<()> {
        self.config = config;
        self.stopwatch = std::time::Instant::now();
        let mut reader: Box<dyn Read> = match self.config.file_path.as_str() {
            "/dev/stdin" => Box::new(io::stdin()),
            _ => Box::new(std::fs::File::open(&self.config.file_path)?),
        };
        let mut writer: Box<dyn Write> = match cfg!(debug_assertions) {
            true => Box::new(std::fs::OpenOptions::new().write(true).open("/dev/null")?),
            false => Box::new(io::stdout()),
        };
        let mut buffer = vec![0; self.config.buffer_size as usize];
        let mut can_update: bool = false;
        let mut last_update = std::time::Instant::now();
        terminal.clear()?;
        while !self.exit {
            // Update the terminal
            if can_update {
                terminal.draw(|frame| self.render_frame(frame))?;
            }

            // Handle rate limiting
            match self.config.rate_limit {
                Some(rate) if self.get_transfer_rate() > rate => continue,
                _ => (),
            }

            // Transfer data
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                self.exit = true;
            }
            self.read_bytes += bytes_read as u64;
            writer.write_all(&buffer[..bytes_read])?;
            writer.flush()?;

            // Calculate ETA
            if can_update && self.stopwatch.elapsed() > std::time::Duration::from_secs(ETA_DELAY_SECONDS) {
                match self.config.size {
                    Some(size) => {
                        self.percent_complete = Some((self.read_bytes as f64 / size as f64) * 100.0);
                        let rate = self.get_transfer_rate();
                        let remaining = match self.read_bytes >= size {
                            true => {
                                0
                            }
                            false => size - self.read_bytes,
                        };
                        self.eta = Some(match rate {
                            0 => std::time::Duration::new(0, 0),
                            _ => std::time::Duration::from_secs(remaining / rate),
                        });
                    }
                    None => (),
                }
            }

            // TODO: replace with match
            // Handle update timing
            if can_update {
                can_update = false;
                if self.stopwatch.elapsed() > std::time::Duration::from_secs(ETA_DELAY_SECONDS) {
                    // Don't start sampling until some time has passed
                    self.samples.push(self.get_transfer_rate());
                }
            } else {
                if last_update.elapsed().as_secs() >= 1 {
                    can_update = true;
                    last_update = std::time::Instant::now();
                }
            };
        }
        terminal.clear()?;
        Ok(())
    }
}

impl Default for NCPV {
    fn default() -> Self {
        Self {
            percent_complete: None,
            stopwatch: std::time::Instant::now(),
            start_time: SystemTime::now(),
            eta: None,
            samples: Vec::new(),
            read_bytes: 0,
            exit: false,
            config: Config::default(),
        }
    }
}

impl Widget for &NCPV {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let start_time_fmt = DateTime::<Local>::from(self.start_time);
        let elapsed_time_fmt = seconds2human(self.stopwatch.elapsed().as_secs()) + " elapsed";
        let option_finish_time = self.eta.map(|eta| Local::now() + chrono::Duration::from_std(eta).unwrap());
        let data_points = Vec::from_iter(self.samples.iter().enumerate().map(|(i, &y)| (i as f64, y as f64)));
        let sample_length = match self.eta {
            Some(eta) => self.stopwatch.elapsed().as_secs() as f64 + eta.as_secs() as f64,
            None => self.stopwatch.elapsed().as_secs() as f64,
        };
        let fastest_speed = data_points.iter().cloned().fold(0.0f64, |acc: f64, (_, y)| acc.max(y));
        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .data(&data_points)
        ];
        let title = match self.config.size {
            Some(size) => format!("  NCurses Pipe Viewer -- {} ({} / {}, {}%) {}/s  ",
                                  self.config.file_path,
                                  bytes2human(self.read_bytes),
                                  bytes2human(size),
                                  self.percent_complete.unwrap_or(0.0).round(),
                                  bytes2human(self.get_transfer_rate())
            ),
            None => format!("  NCurses Pipe Viewer -- {} {}/s  ", self.config.file_path,
                            bytes2human(self.get_transfer_rate())),
        };
        let divisions: u64 = (area.height / 5).into();
        let step = fastest_speed as u64 / divisions;
        let labels_y = (0..=divisions).map(|i|
            bytes2human(i * step).bold()).collect::<Vec<_>>();
        Chart::new(datasets)
            .block(Block::bordered().title(title.bold()))
            .x_axis(
                Axis::default()
                    .title(self.eta.map_or_else(|| seconds2human(
                        self.stopwatch.elapsed().as_secs()),
                                                |eta| self.get_eta_text(eta)
                    ))
                    .labels(vec![
                        start_time_fmt.to_string().into(),
                        elapsed_time_fmt.into(),
                        option_finish_time.map_or_else(|| "".into(), |finish_time| finish_time.to_string().into()),
                    ])
                    .bounds([0.0, sample_length + (sample_length * 0.05)]),
            )
            .y_axis(
                Axis::default()
                    .title(format!("{}/s", bytes2human_scale(self.get_transfer_rate())))
                    .labels(labels_y)
                    .bounds([0.0, fastest_speed]),
            )
            .render(area, buf);
    }
}
