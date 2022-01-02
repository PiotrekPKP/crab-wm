use std::process::exit;

use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, EventMask, GrabMode, KeyButMask};
use crate::errors::ANOTHER_WM_RUNNING;

pub struct Crab<'a, C: Connection> {
    connection: &'a C,
    screen_num: usize,
}

impl<'a, C: Connection> Crab<'a, C> {
    pub fn new(connection: &'a C, screen_num: usize) -> Result<Self, &'a C> {
        let screen = &connection.setup().roots[screen_num];

        let change_values = ChangeWindowAttributesAux::default().event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT);

        let changed_values = &connection.change_window_attributes(screen.root, &change_values).unwrap().check();
        if let Err(ReplyError::X11Error(_)) = changed_values {
            eprintln!("{}", ANOTHER_WM_RUNNING);
            exit(1);
        }

        Ok(Self {
            connection: &connection,
            screen_num,
        })
    }

    pub fn event_loop(&self) {
        loop {
            let _ = self.connection.flush();

            if let Ok(event) = self.connection.wait_for_event() {
                match event {
                    _ => println!("Event {:?} dispatched!", event),
                }
            }
        }
    }
}