use std::cmp::Reverse;

use x11rb::connection::Connection;
use x11rb::{COPY_DEPTH_FROM_PARENT, CURRENT_TIME};
use x11rb::protocol::xproto::{ConfigureRequestEvent, ConfigureWindowAux, ConnectionExt, CreateWindowAux, EnterNotifyEvent, EventMask, ExposeEvent, GetGeometryReply, InputFocus, LeaveNotifyEvent, MapRequestEvent, MotionNotifyEvent, SetMode, StackMode, UnmapNotifyEvent, Window, WindowClass};

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
            .sequences_to_ignore
            .push(Reverse(cookie.sequence_number()))
    }

    fn handle_unmap_notify_event(&self, state: &mut CrabState<C>, event: UnmapNotifyEvent) {
        let root = state.connection.setup().roots[state.screen_num].root;

        state.window_states.retain(|window_state| {
            if window_state.window != event.window { return true; }

            state.connection.change_save_set(SetMode::DELETE, window_state.window).unwrap();
            state.connection.reparent_window(window_state.window, root, window_state.x, window_state.y).unwrap();
            state.connection.destroy_window(window_state.frame_window).unwrap();

            return false;
        })
    }

    fn handle_configure_request_event(&self, state: &mut CrabState<C>, event: ConfigureRequestEvent) {
        if let Some(_) = state.find_window_state(event.window) {
            unimplemented!();
        }

        let configure_aux = ConfigureWindowAux::from_configure_request(&event)
            .sibling(None)
            .stack_mode(None);

        state.connection.configure_window(event.window, &configure_aux).unwrap();
    }

    fn handle_map_request_event(&self, state: &mut CrabState<C>, event: MapRequestEvent) {
        self.manage_window(
            state,
            event.window,
            &state.connection.get_geometry(event.window).unwrap().reply().unwrap()
        );
    }

    fn handle_expose_event(&self, state: &mut CrabState<C>, event: ExposeEvent) {
        state.pending_exposes.push(event.window);
    }

    fn handle_enter_notify_event(&self, state: &mut CrabState<C>, event: EnterNotifyEvent) {
        let window_state = state.find_window_state(event.event);

        if !window_state.is_some() { return; }
        let window_state = window_state.unwrap();

        state.connection.set_input_focus(
            InputFocus::PARENT,
            window_state.window,
            CURRENT_TIME
        ).unwrap();

        let configure_aux = ConfigureWindowAux::new().stack_mode(StackMode::ABOVE);
        state.connection.configure_window(window_state.frame_window, &configure_aux).unwrap();
    }

    fn handle_leave_notify_event(&self, state: &mut CrabState<C>, _event: LeaveNotifyEvent) {
        state.connection.set_input_focus(
            InputFocus::PARENT,
            0 as u32,
            CURRENT_TIME
        ).unwrap();
    }

    fn handle_motion_notify_event(&self, state: &mut CrabState<C>, event: MotionNotifyEvent) {
        let drag_state = &state.drag_window;

        if !drag_state.is_some() { return; }
        let drag_state = drag_state.unwrap();

        let x: i32 = (drag_state.x + event.root_x).into();
        let y: i32 = (drag_state.y + event.root_y).into();

        let configure_aux = ConfigureWindowAux::new().x(x).y(y);
        state.connection.configure_window(drag_state.window.window, &configure_aux).unwrap();
    }
}