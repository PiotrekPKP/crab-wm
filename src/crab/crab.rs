use std::process::{Command, exit};

use log::{info, LevelFilter};
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, EventMask};

use crate::crab::crab_state::CrabState;
use crate::errors::ANOTHER_WM_RUNNING;
use crate::layout::floating_layout::FloatingLayout;

pub struct Crab<'a, C: Connection> {
    connection: &'a C,
    state: CrabState<'a, C>,
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
            state,
        })
    }

    pub fn event_loop(&mut self) {
        loop {
            self.state.flush();
            let _ = self.connection.flush();

            let event = self.connection.wait_for_event().unwrap();
            let mut event_opt = Some(event);

            while let Some(event) = event_opt {
                if let Event::ClientMessage(_) = event {
                    return;
                }
                println!("{:?}", event);

                self.state.handle_event(event);

                event_opt = self.connection.poll_for_event().unwrap();
            }
        }
    }
}