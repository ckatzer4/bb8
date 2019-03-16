extern crate bb8;
extern crate bb8_tiberius;
extern crate futures;
extern crate futures_state_stream;
extern crate tiberius;

use bb8_tiberius::TiberiusConnectionManager;

use futures::future::Future;
use futures_state_stream::StateStream;

type Cnxn = tiberius::SqlConnection<Box<dyn tiberius::BoxableIo>>;
fn select_one(
    cnxn: Cnxn,
) -> impl Future<Item = (Vec<i32>, Option<Cnxn>), Error = (tiberius::Error, Option<Cnxn>)> {
    cnxn.simple_query("SELECT 1")   // perform a simple "SELECT 1"
        .map(|row| row.get(0))      // extract first column from each row
        .collect()                  // convert to a Vec<i32>
        .map(|(v, c)| (v, Some(c))) // on success, return the connection to be passed back to the pool
        .map_err(|e| (e, None))     // on error, return None to drop a potentially bad connection
}

fn main() {
    let pool = bb8::Pool::builder().build(TiberiusConnectionManager::new(
        "server=localhost:1433;user=auth;password=auth;",
    ));
    let one: Result<Vec<i32>, _> = pool
        .and_then(|pool| {
            pool.run(|c| {
                if let Some(c) = c {
                    select_one(c).wait()
                } else {
                    // this shouldn't happen with a healthy connection pool, but this is rust so we
                    // handle potential errors
                    Err((tiberius::Error::Canceled, None))
                }
            })
        })
        .wait();
    println!("{:?}", one);
}


