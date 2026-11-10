use rusqlite::Connection;

use crate::db::{connection::open_memory_database, migrations::run_migrations};

pub fn migrated_memory_database() -> Connection {
    let mut connection = open_memory_database().expect("in-memory database opens");
    run_migrations(&mut connection).expect("migrations succeed");
    connection
}
