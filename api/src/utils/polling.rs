use crate::routes::ApiResult;

use event_listener::Event;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static! {
    static ref EVENTS: Arc<RwLock<HashMap<u64, Arc<Event>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref LISTENERS: Arc<RwLock<HashMap<u64, u32>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn notify(guild: u64) -> ApiResult<()> {
    let events_arc = EVENTS.clone();
    let events = events_arc.read()?;

    if let Some(event) = events.get(&guild) {
        let event = event.clone();
        event.notify(usize::MAX);
    }

    Ok(())
}

pub async fn listen(guild: u64) -> ApiResult<()> {
    let listeners_arc = LISTENERS.clone();
    let events_arc = EVENTS.clone();

    let mut listeners = listeners_arc.write()?;
    let listener = listeners.entry(guild).or_insert(0);
    *listener += 1;
    drop(listeners);

    let mut events = events_arc.write()?;
    let event = events
        .entry(guild)
        .or_insert_with(|| Arc::new(Event::new()))
        .clone();
    drop(events);

    event.listen().await;

    let mut listeners = listeners_arc.write()?;
    let listener = listeners.entry(guild).or_insert(1);

    *listener -= 1;

    if *listener == 0 {
        listeners.remove(&guild);

        let mut events = events_arc.write()?;
        events.remove(&guild);
    }

    Ok(())
}
