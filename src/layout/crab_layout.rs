use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConfigureRequestEvent, EnterNotifyEvent, ExposeEvent, GetGeometryReply, LeaveNotifyEvent, MapRequestEvent, MotionNotifyEvent, UnmapNotifyEvent, Window};

use crate::crab::crab_state::CrabState;

pub trait CrabLayout<C: Connection> {
    fn manage_window(&self, state: &mut CrabState<C>, window: Window, geometry: &GetGeometryReply);

    fn handle_unmap_notify_event(&self, state: &mut CrabState<C>, event: UnmapNotifyEvent);
    fn handle_configure_request_event(&self, state: &mut CrabState<C>, event: ConfigureRequestEvent);
    fn handle_map_request_event(&self, state: &mut CrabState<C>, event: MapRequestEvent);
    fn handle_expose_event(&self, state: &mut CrabState<C>, event: ExposeEvent);
    fn handle_enter_notify_event(&self, state: &mut CrabState<C>, event: EnterNotifyEvent);
    fn handle_leave_notify_event(&self, state: &mut CrabState<C>, event: LeaveNotifyEvent);
    fn handle_motion_notify_event(&self, state: &mut CrabState<C>, event: MotionNotifyEvent);
}