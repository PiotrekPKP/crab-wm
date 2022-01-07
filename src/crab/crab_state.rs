use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::error::Error;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{Atom, ConnectionExt, CreateGCAux, Gcontext, GetGeometryReply, MapState, Window};

use crate::layout::crab_layout::CrabLayout;

#[derive(Copy, Clone)]
pub enum DragType {
    Resize,
    Move,
}

#[derive(Copy, Clone)]
pub struct DragState {
    pub window: WindowState,
    pub drag_type: DragType,
    pub x: i16,
    pub y: i16,
}

impl DragState {
    pub fn new(window: WindowState, drag_type: DragType, x: i16, y: i16) -> Self {
        Self {
            window,
            drag_type,
            x,
            y,
        }
    }
}

#[derive(Copy, Clone)]
pub struct WindowState {
    pub window: Window,
    pub frame_window: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
}

impl WindowState {
    pub fn new(window: Window, frame_window: Window, geometry: &GetGeometryReply) -> Self {
        Self {
            window,
            frame_window,
            x: geometry.x,
            y: geometry.y,
            width: geometry.width,
        }
    }
}

pub struct CrabState<'a, C: Connection> {
    pub connection: &'a C,
    pub layout: &'a dyn CrabLayout<C>,
    pub screen_num: usize,
    pub black_gc: Gcontext,
    pub wm_protocols: Atom,
    pub wm_delete_window: Atom,
    pub pending_exposes: Vec<Window>,
    pub window_states: Vec<WindowState>,
    pub sequences_to_ignore: BinaryHeap<Reverse<u64>>,
    pub drag_window: Option<DragState>,
}

impl<'a, C: Connection> CrabState<'a, C> {
    pub fn new(connection: &'a C, layout: &'a dyn CrabLayout<C>, screen_num: usize) -> Result<CrabState<'a, C>, Box<dyn Error>> {
        let screen = &connection.setup().roots[screen_num];
        let black_gc = connection.generate_id().unwrap();

        let gc_aux = CreateGCAux::default()
            .background(screen.white_pixel)
            .foreground(screen.black_pixel);

        connection.create_gc(black_gc, screen.root, &gc_aux).unwrap();

        let wm_protocols = connection.intern_atom(false, b"WM_PROTOCOLS").unwrap();
        let wm_delete_window = connection.intern_atom(false, b"WM_DELETE_WINDOW").unwrap();

        Ok(Self {
            connection,
            layout,
            screen_num,
            black_gc,
            wm_protocols: wm_protocols.reply()?.atom,
            wm_delete_window: wm_delete_window.reply()?.atom,
            pending_exposes: Vec::new(),
            window_states: Vec::new(),
            sequences_to_ignore: BinaryHeap::new(),
            drag_window: None
        })
    }

    pub fn map_windows(&mut self) {
        let screen = &self.connection.setup().roots[self.screen_num];

        self
            .connection
            .query_tree(screen.root)
            .unwrap()
            .reply()
            .unwrap()
            .children
            .iter()
            .for_each(|window| {
                let geometry = self.connection.get_geometry(*window).unwrap();
                let attributes = self.connection.get_window_attributes(*window).unwrap();

                let (attributes, geometry) = (attributes.reply(), geometry.reply());

                if attributes.is_ok() && geometry.is_ok() {
                    if !attributes.as_ref().unwrap().override_redirect && attributes.unwrap().map_state != MapState::UNMAPPED {
                        self.layout.manage_window(self, *window, &geometry.unwrap());
                    }
                }
            })
    }

    pub fn handle_event(&mut self, event: Event) {
        let mut should_ignore_event = false;

        if let Some(sequence) = event.wire_sequence_number() {
            while let Some(&Reverse(to_ignore)) = self.sequences_to_ignore.peek() {
                if to_ignore.wrapping_sub(sequence as u64) <= u64::MAX / 2 {
                    should_ignore_event = to_ignore == sequence as u64;
                    break;
                }
            }
        }

        if should_ignore_event { return; }

        match event {
            Event::UnmapNotify(event) => self.layout.handle_unmap_notify_event(self, event),
            Event::ConfigureRequest(event) => self.layout.handle_configure_request_event(self, event),
            Event::MapRequest(event) => self.layout.handle_map_request_event(self, event),
            Event::Expose(event) => self.layout.handle_expose_event(self, event),
            Event::EnterNotify(event) => self.layout.handle_enter_notify_event(self, event),
            Event::LeaveNotify(event) => self.layout.handle_leave_notify_event(self, event),
            Event::MotionNotify(event) => self.layout.handle_motion_notify_event(self, event),
            _ => {}
        }
    }

    pub fn find_window_state(&self, window: Window) -> Option<&WindowState> {
        self
            .window_states
            .iter()
            .find(|window_state| window == window_state.window || window == window_state.frame_window)
    }

    pub fn flush(&mut self) {
        self
            .pending_exposes
            .clone()
            .iter()
            .enumerate()
            .for_each(|(i, _pending_expose)| {
                self.pending_exposes.remove(i);
            });
    }
}