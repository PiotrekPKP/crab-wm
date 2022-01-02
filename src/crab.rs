use x11rb::*;
use x11rb::connection::Connection;

pub struct Crab<C: Connection> {
    connection: C,
}

impl<C: Connection> Crab<C> {
    pub fn new(connection: C) -> Crab<C> {
        Crab {
            connection
        }
    }
}