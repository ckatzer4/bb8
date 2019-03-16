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

/// Connection manager for `tiberius::SqlConnection`s
///
/// # Example:
/// ```rust,no_run
/// extern crate bb8;
/// extern crate tiberius;
/// use bb8_tiberius::TiberiusConnectionManager;
///
/// let pool = bb8::Pool::builder().build(TiberiusConnectionManager::new(
///         "server=localhost:1433;user=auth;password=auth;",
///     ));
/// // use pool, see /examples/ directory in repository
/// ```
pub struct TiberiusConnectionManager {
    connection_string: String,
}

impl TiberiusConnectionManager {
    /// Create a new `TiberiusConnectionManager` using the given `connection_string`.
    ///
    /// See the [Tiberius README] for more details about parameters in the connection string.
    ///
    /// # Example:
    /// ```rust
    /// extern crate bb8;
    /// extern crate tiberius;
    /// use bb8_tiberius::TiberiusConnectionManager;
    ///
    /// let cnxm_mgr = TiberiusConnectionManager::new(
    ///         "server=localhost:1433;user=auth;password=auth;"
    ///     );
    /// ```
    ///
    ///
    /// [Tiberius README]: https://github.com/steffengy/tiberius#supported-connection-string-parameters
    pub fn new<S: std::string::ToString>(connection_string: S) -> TiberiusConnectionManager {
        TiberiusConnectionManager {
            connection_string: connection_string.to_string(),
        }
    }
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
