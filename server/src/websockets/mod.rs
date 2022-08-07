use serde::Serialize;
use crate::utils::music::Track;

#[derive(Debug, Serialize)]
pub enum Loop {
    None, 
    Track, 
    Queue,
}

#[derive(Debug, Serialize)]
pub struct UpdateTrack {
    current_track: i64, 
    position: u32, // number of seconds since the start 
    play: bool, 
    looping: Loop,
}

#[derive(Debug, Serialize)]
pub struct UpdateQueue {
    new_queue: Vec<Track>,
}

#[derive(Debug, Serialize)]
pub enum Kind {
    UpdateQueue, 
    UpdatePlayer, 
}

#[derive(Debug, Serialize)]
pub enum Data {
    UpdateTrack(UpdateTrack),
    UpdateQueue(UpdateQueue),
}

#[derive(Debug, Serialize)]
pub struct General {
    kind: Kind,
    data: Data 
}

impl UpdateTrack {
    
}