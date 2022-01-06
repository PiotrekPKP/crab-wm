use x11rb::connection::Connection;
use x11rb::protocol::xproto::{GetGeometryReply, Window};
use crate::crab::crab_state::CrabState;

pub trait CrabLayout<C: Connection> {
    fn manage_window(&self, state: &mut CrabState<C>, window: Window, geometry: &GetGeometryReply);
}