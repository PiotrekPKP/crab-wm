use x11rb::connect;
use crab_wm::crab::Crab;

fn main() {
    let (connection, screen_num) = connect(None).unwrap();

    if let Ok(crab) = Crab::new(&connection, screen_num) {
        crab.event_loop();
    }
    else {
        panic!("CrabWM could not start!");
    }
}
