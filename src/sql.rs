use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use once_cell::sync::OnceCell;
use rusqlite::{params, Connection};

use log::info;

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
            .execute(TableRow::init_row(), [])
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
                ))
            })
            .expect(format!("query error without timestamp").as_str())
            .map(|x| x.unwrap())
            .collect()
        }
    }
}

#[derive(Debug)]
pub struct TableRow {
    name: String,
    timestamp: usize,
    content: String,
    time: String,
}
impl TableRow {
    pub fn new(name: String, timestamp: usize, content: String, time: String) -> Self {
        Self {
            name,
            timestamp,
            content,
            time,
        }
    }

    pub fn to_msg(&self) -> String {
        format!("time: ")
    }

    const fn init_row() -> &'static str {
        "CREATE TABLE msg(name TEXT, timestamp INTEGER, content TEXT, time TEXT);"
    }

    const fn insert_row() -> &'static str {
        "INSERT INTO msg (name, timestamp, content, time) VALUES (?1, ?2, ?3, ?4);"
    }
}

pub fn insert_1(msg: TableRow) {
    conn()
        .lock()
        .expect("get mutex lock error")
        .execute(
            TableRow::insert_row(),
            params![msg.name, msg.timestamp, msg.content, msg.time],
        )
        .unwrap();
}

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
        insert_1(TableRow::new(
            "123".to_string(),
            1,
            "456".to_string(),
            "time is what".to_string(),
        ));

        println!("{:?}", query_10(None))
    }
}
