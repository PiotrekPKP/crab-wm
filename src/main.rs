use std::process::{Command, exit};

use x11rb::connect;
use crab_wm::crab::crab::Crab;

use crab_wm::errors::COULD_NOT_INITIALIZE;

fn main() {
    let (connection, screen_num) = connect(None).unwrap();

    if let Ok(crab) = &mut Crab::new(&connection, screen_num) {
        crab.event_loop();
        Command::new("alacritty").spawn().unwrap();
    } else {
        eprintln!("{}", COULD_NOT_INITIALIZE);
        exit(1);
    }
}
