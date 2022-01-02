use x11rb::connect;
use crab_wm::crab::Crab;

fn main() {
    let (connection, screen_num) = connect(None).unwrap();

    let crab = Crab::new(connection);
}
