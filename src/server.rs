//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use log::{error, info};

use actix::prelude::*;

use crate::{
    id, local_time,
    sql::{insert_1, TableRow},
};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// room name
    pub room_name: String,
    /// timestamp
    // pub timestamp: usize,
    /// msg content
    pub content: String,
    /// sender name
    pub name: String,
    /// time
    pub time: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client ID
    pub id: usize,
    /// Room name
    pub room_name: String,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    id_allocator: Arc<Mutex<id::Allocator>>,
    timestamp: usize,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            id_allocator: Arc::new(Mutex::new(id::Allocator::new())),
            timestamp: 0,
        }
    }
}

impl ChatServer {
    /// Send message to all users in the room
    fn send_for_all(&self, room_name: &str, content: &str) {
        if let Some(sessions) = self.rooms.get(room_name) {
            for id in sessions {
                if let Some(addr) = self.sessions.get(id) {
                    addr.do_send(Message(content.to_owned()));
                }
            }
        }
    }

    fn stamp(&mut self) -> usize {
        self.timestamp += 1;
        self.timestamp
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _cx: &mut Context<Self>) -> Self::Result {
        // notify all users in same room
        self.send_for_all("main", "someone joined");

        // register session with random id

        let id = match self.id_allocator.lock() {
            Ok(mut v) => v.get(),
            Err(e) => {
                error!("mutex lock error: {e}");
                return 0;
            }
        };

        info!("id: {id} joined.");

        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.rooms
            .entry("main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("id: {} disconnected.", msg.id);

        match self.id_allocator.lock() {
            Ok(mut v) => v.remove(msg.id),
            Err(e) => error!("mutex lock error: {e}"),
        }

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_for_all(&room, "someone disconnected");
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        info!("received msg from user. id: {}", msg.id);
        let table_row = TableRow::new(msg.name, self.stamp(), msg.content, msg.time, 0);

        insert_1(&table_row);

        self.send_for_all("main", &table_row.to_msg());
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, room_name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }

        let table_row_disconnect = TableRow::new(
            "system".to_string(),
            self.stamp(),
            format!("id: {id} disconnected"),
            local_time::get(),
            2,
        );
        insert_1(&table_row_disconnect);

        // send message to other users
        for room in rooms {
            self.send_for_all(&room, &table_row_disconnect.to_msg());
        }

        self.rooms
            .entry(room_name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        let table_row_connect = TableRow::new(
            "system".to_string(),
            self.stamp(),
            format!("id: {id} connected"),
            local_time::get(),
            1,
        );
        insert_1(&table_row_connect);

        self.send_for_all(&room_name, &table_row_connect.to_msg());
    }
}
