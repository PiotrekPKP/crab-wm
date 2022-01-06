use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::error::Error;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, ConnectionExt, CreateGCAux, Gcontext, GetGeometryReply, MapState, Window};

use crate::layout::crab_layout::CrabLayout;

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
    pub sources_to_ignore: BinaryHeap<Reverse<u64>>
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
            sources_to_ignore: BinaryHeap::new()
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