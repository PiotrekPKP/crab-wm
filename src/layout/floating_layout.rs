use std::cmp::Reverse;

use x11rb::connection::Connection;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::protocol::xproto::{ConnectionExt, CreateWindowAux, EventMask, GetGeometryReply, SetMode, Window, WindowClass};

use crate::crab::crab_state::{CrabState, WindowState};
use crate::layout::crab_layout::CrabLayout;

pub struct FloatingLayout;

impl<C: Connection> CrabLayout<C> for FloatingLayout {
    fn manage_window(&self, state: &mut CrabState<C>, window: Window, geometry: &GetGeometryReply) {
        let connection = state.connection;
        let screen = &connection.setup().roots[state.screen_num];

        let frame_window = state.connection.generate_id().unwrap();

        let event_masks = EventMask::EXPOSURE
            | EventMask::SUBSTRUCTURE_NOTIFY
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE
            | EventMask::POINTER_MOTION
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW;

        let window_aux = CreateWindowAux::new().event_mask(event_masks);

        connection.create_window(
            COPY_DEPTH_FROM_PARENT,
            frame_window,
            screen.root,
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
            1,
            WindowClass::INPUT_OUTPUT,
            0,
            &window_aux,
        ).unwrap();

        connection.grab_server().unwrap();
        connection.change_save_set(SetMode::INSERT, window).unwrap();

        let cookie = connection.reparent_window(window, frame_window, 0, 0).unwrap();

        connection.map_window(window).unwrap();
        connection.map_window(frame_window).unwrap();

        connection.ungrab_server().unwrap();

        state
            .window_states
            .push(WindowState::new(window, frame_window, geometry));

        state
            .sources_to_ignore
            .push(Reverse(cookie.sequence_number()))
    }
}