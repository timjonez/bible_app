use sqlite;
use sqlite::{Connection, Error};

pub struct Database {
    pub connection: Connection
}

impl Database {
    pub fn new() -> Result<Database, Error> {
        let conn = sqlite::open("./db.sqlite").unwrap();

        let db = Database {
            connection: conn
        };

        Ok(db)
    }
}