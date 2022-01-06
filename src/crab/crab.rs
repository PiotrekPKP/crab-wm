use log::{info, LevelFilter};
use std::process::exit;
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, EventMask};
use crate::crab::crab_state::CrabState;

use crate::errors::ANOTHER_WM_RUNNING;
use crate::layout::floating_layout::FloatingLayout;
use crate::logger::log_to_file;

pub struct Crab<'a, C: Connection> {
    connection: &'a C,
    screen_num: usize,
    state: CrabState<'a, C>
}

impl<'a, C: Connection> Crab<'a, C> {
    pub fn new(connection: &'a C, screen_num: usize) -> Result<Self, &'a C> {
        simple_logging::log_to_file("crab.log", LevelFilter::Debug).unwrap();
        info!("Logging has been initialized.");

        let screen = &connection.setup().roots[screen_num];

        let event_masks = EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT | EventMask::BUTTON_PRESS | EventMask::KEY_PRESS;
        let change_values = ChangeWindowAttributesAux::default().event_mask(event_masks);

        let changed_values = &connection.change_window_attributes(screen.root, &change_values).unwrap().check();
        if let Err(ReplyError::X11Error(_)) = changed_values {
            eprintln!("{}", ANOTHER_WM_RUNNING);
            exit(1);
        }

        let mut state = CrabState::new(connection, &FloatingLayout {}, screen_num).unwrap();
        state.map_windows();

        Ok(Self {
            connection: &connection,
            screen_num,
            state
        })
    }

    pub fn event_loop(&mut self) {
        loop {
            self.state.flush();
            let _ = self.connection.flush();

            if let Ok(event) = self.connection.wait_for_event() {
                match event {
                    _ => log_to_file(format!("Got event {:?}", event), LevelFilter::Debug)
                }
            }
        }
    }
}