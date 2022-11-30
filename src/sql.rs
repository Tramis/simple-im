use std::sync::{Arc, Mutex};

use once_cell::sync::OnceCell;
use rusqlite::{params, Connection};

use log::info;
use serde::{Deserialize, Serialize};

pub fn set_path<T: AsRef<str>>(val: T) {
    let real_path = unsafe { &mut *(path() as *const _ as *mut Option<String>) };

    real_path.replace(val.as_ref().to_string());
}

pub fn path() -> &'static Option<String> {
    static PATH: OnceCell<Option<String>> = OnceCell::new();
    PATH.get_or_init(|| None)
}

pub fn conn() -> Arc<Mutex<Connection>> {
    static CONN: OnceCell<Arc<Mutex<Connection>>> = OnceCell::new();
    CONN.get_or_init(|| {
        Arc::new(Mutex::new(
            Connection::open(path().as_ref().expect("no specified sqlite file"))
                .expect("sqlite connect error"),
        ))
    })
    .clone()
}

pub fn init() {
    if let Some(path) = path() {
        // create or open file
        info!("init sqlite file: {path}");
        std::fs::File::create(path).unwrap();

        conn()
            .lock()
            .expect("get mutex lock error")
            .execute(TableRow::init_table(), [])
            .expect("init table error");
    }
}

/// # query 10 msg from table
///
/// if `timestamp` == `None`, query from last,
/// otherwise query ahead `timestamp`
///
/// return the last timestamp
pub fn query_10(timestamp: Option<usize>) -> Vec<TableRow> {
    if path().is_none() {
        return vec![];
    }

    let conn = conn();
    let conn = conn.lock().expect("get mutex lock error");
    match timestamp {
        Some(timestamp) => {
            let mut stmt = conn
                .prepare("SELECT * FROM msg WHERE timestamp < ? ORDER BY timestamp DESC LIMIT 10")
                .expect("cook sql statement error");
            stmt.query_map(params![timestamp], |row| {
                Ok(TableRow::new(
                    row.get(0).expect("get name error"),
                    row.get(1).expect("get timestamp error"),
                    row.get(2).expect("get content error"),
                    row.get(3).expect("get time error"),
                    Status::RollBack as i8
                ))
            })
            .expect(format!("query error with timestamp: {}", timestamp).as_str())
            .map(|x| x.unwrap())
            .collect()
        }
        None => {
            let mut stmt = conn
                .prepare("SELECT * FROM msg ORDER BY timestamp DESC LIMIT 10")
                .expect("cook sql statement error");
            stmt.query_map([], |row| {
                Ok(TableRow::new(
                    row.get(0).expect("get name error"),
                    row.get(1).expect("get timestamp error"),
                    row.get(2).expect("get content error"),
                    row.get(3).expect("get time error"),
                    Status::RollBack as i8
                ))
            })
            .expect(format!("query error without timestamp").as_str())
            .map(|x| x.unwrap())
            .collect()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableRow {
    name: String,
    timestamp: usize,
    content: String,
    time: String,
    status: i8
}
impl TableRow {
    pub fn new(name: String, timestamp: usize, content: String, time: String, status: i8) -> Self {
        Self {
            name,
            timestamp,
            content,
            time,
            status
        }
    }

    pub fn to_msg(&self) -> String {
        serde_json::ser::to_string(self).expect("json to string error")
    }

    const fn init_table() -> &'static str {
        "CREATE TABLE msg(name TEXT, timestamp INTEGER, content TEXT, time TEXT);"
    }

    const fn insert_row() -> &'static str {
        "INSERT INTO msg (name, timestamp, content, time) VALUES (?1, ?2, ?3, ?4);"
    }
}

pub fn insert_1(row: &TableRow) {
    if path().is_none() {
        return;
    }

    conn()
        .lock()
        .expect("get mutex lock error")
        .execute(
            TableRow::insert_row(),
            params![row.name, row.timestamp, row.content, row.time],
        )
        .unwrap();
}
use crate::status::Status;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conn() {
        set_path("./test.sqlite");

        let conn = conn();
    }

    #[test]
    fn test_query() {
        set_path("./test.sqlite");

        init();
        insert_1(&TableRow::new(
            "123".to_string(),
            1,
            "456".to_string(),
            "time is what".to_string(),
            Status::Normal as i8
        ));

        println!("{:?}", query_10(None))
    }

    #[test]
    fn test_query_10() {
        set_path("./test.sqlite");

        init();

        for i in 0..20 {
            insert_1(&TableRow::new(
                "anonymous".to_string(),
                i,
                format!("any content: {i}"),
                "time is what".to_string(),
                Status::Normal as i8
            ));
        }

        println!("{:#?}", query_10(None));
        println!("{:#?}", query_10(Some(1)))
    }
}
