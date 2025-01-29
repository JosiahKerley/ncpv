mod ncpv;
mod tui;
mod utils;
mod cli;

use std::io;
use ncpv::NCPV;

fn main() -> io::Result<()> {
    let config = cli::getopts();
    let mut terminal = tui::init()?;
    let exit_code = NCPV::default().run(&mut terminal, config);
    tui::restore()?;
    exit_code
}
