use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use std::borrow::Borrow;
use crate::utils::player::Player;
use crate::utils::queue::Queue;
use actix::prelude::*;
use diesel::dsl::Update;
use rand::{self, rngs::ThreadRng, Rng};
use rand::distributions::uniform::SampleBorrow;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

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
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
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
    pub name: String,
}



/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<i64, HashSet<usize>>,
    rng: ThreadRng,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert(0, HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
        }
    }
}

impl ChatServer {
    /// Send message to all users in the room
    fn send_message(&self, room: &i64, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
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

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        self.send_message(&0i64, "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.rooms
            .entry(0i64)
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&0i64 , &format!("ok"), 0);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

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
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str(), msg.id);
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
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&name, "Someone connected", id);
    }
}

use serde::Serialize;
use crate::utils;
use crate::utils::music::Track;
/*
#[derive(Debug, Serialize, Copy, Clone)]
pub enum Loop {
    None,
    Track,
    Queue,
} */
/*
#[derive(Message)]
#[rtype(result = "()")]
#[derive(Debug, Serialize)]
pub struct UpdatePlayer {
    pub current_track: i64,
    pub position: u32, // number of seconds since the start
    pub paused: bool,
    pub looping: Loop,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Debug, Serialize)]
pub struct UpdateQueue {
    pub new_queue: Vec<Track>,
}
*/

#[derive(Debug, Serialize)]
pub enum Kind {
    UpdateQueue,
    UpdatePlayer,
}

#[derive(Debug, Serialize)]
pub enum UpdateData {
    UpdatePlayer(Player),
    UpdateQueue(Queue),

}
#[derive(Message)]
#[rtype(result = "()")]
#[derive(Debug, Serialize)]
pub struct General {
    pub kind: Kind,
    pub data: UpdateData
}
impl Handler<General> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: General, _: &mut Context<Self>) {
        let json = serde_json::to_string(&msg).unwrap();
        match msg {
            General { kind: Kind::UpdatePlayer, data: UpdateData::UpdatePlayer(dat) } => {

                self.send_message(dat.space, &json, 0);
            },
            General { kind: Kind::UpdateQueue, data: UpdateData::UpdateQueue(dat) } => {
                self.send_message(dat.space, &json, 0);
            },
            _ => {}
        }
        self.send_message(, &json, 0);
    }
}
/*
impl ChatServer {
    fn update_queue(&self, new_queue: Vec<Track>, ) {
        let update = General {
            kind: Kind::UpdateQueue,
            data: UpdateData::UpdateQueue(UpdateQueue {
                new_queue
            })
        };

        let json = serde_json::to_string(&update).unwrap();

        self.send_message("main", &json, 0);
    }

    fn update_player(&self, current_track: i64, position: u32, paused: bool, looping: Loop) {
        let update = General {
            kind: Kind::UpdatePlayer,
            data: UpdateData::UpdatePlayer(UpdatePlayer {
                current_track,
                position,
                paused,
                looping
            })
        };

        let json = serde_json::to_string(&update).unwrap();

        self.send_message("main", &json, 0);
    }
}
 */