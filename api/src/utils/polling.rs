use event_listener::Event;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

lazy_static! {
    static ref EVENTS: Arc<RwLock<HashMap<u64, Arc<Event>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref LISTENERS: Arc<RwLock<HashMap<u64, u32>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn notify(guild: u64) {
    let events_arc = EVENTS.clone();
    let events = events_arc.read().unwrap();

    if let Some(event) = events.get(&guild) {
        let event = event.clone();
        event.notify(usize::MAX);
    }
}

pub fn listen(guild: u64, timeout: u64) {
    let listeners_arc = LISTENERS.clone();
    let mut listeners = listeners_arc.write().unwrap();
    let listener = listeners.entry(guild).or_insert(0);
    *listener += 1;

    drop(listeners);

    let events_arc = EVENTS.clone();
    let mut events = events_arc.write().unwrap();
    let event = events
        .entry(guild)
        .or_insert_with(|| Arc::new(Event::new()))
        .clone();

    drop(events);

    event.listen().wait_timeout(Duration::from_millis(timeout));

    let listeners_arc = LISTENERS.clone();
    let mut listeners = listeners_arc.write().unwrap();
    let listener = listeners.entry(guild).or_insert(1);
    *listener -= 1;

    if *listener == 0 {
        listeners.remove(&guild);
        drop(listeners);

        let events_arc = EVENTS.clone();
        let mut events = events_arc.write().unwrap();
        events.remove(&guild);
    }
}
