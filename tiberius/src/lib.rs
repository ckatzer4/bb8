//! Tiberius support for the `bb8` connection pool
#![deny(missing_docs, missing_debug_implementations)]

pub extern crate bb8;
pub extern crate tiberius;

extern crate futures;
extern crate futures_state_stream;

use futures::Future;
use futures_state_stream::StateStream;
use std::fmt;
use std::io;

/// A `bb8::ManageConnection` for `tiberius::SqlConnection`s
pub struct TiberiusConnectionManager {
    connection_string: String,
}

impl bb8::ManageConnection for TiberiusConnectionManager {
    type Connection = Option<tiberius::SqlConnection<Box<dyn tiberius::BoxableIo>>>;
    type Error = tiberius::Error;

    fn connect(
        &self,
    ) -> Box<Future<Item = Self::Connection, Error = Self::Error> + Send + 'static> {
        Box::new(tiberius::SqlConnection::connect(&self.connection_string).map(|c| Some(c)))
    }

    fn is_valid(
        &self,
        conn: Self::Connection,
    ) -> Box<Future<Item = Self::Connection, Error = (Self::Error, Self::Connection)> + Send> {
        let test = conn.unwrap().simple_query("SELECT 1").collect();
        Box::new(test.then(move |r| match r {
            Ok((_, s)) => Ok(Some(s)),
            Err(e) => Err((e, None)),
        }))
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.is_none()
    }

    fn timed_out(&self) -> Self::Error {
        tiberius::Error::Io(io::Error::new(
            io::ErrorKind::TimedOut,
            "TiberiusConnectionManager timed out!",
        ))
    }
}

impl fmt::Debug for TiberiusConnectionManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TiberiusConnectionManager {{ connection_string: {} }}",
            &self.connection_string
        )
    }
}
