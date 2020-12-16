table! {
    account (id) {
        id -> Int8,
        username -> Text,
        discriminator -> Int4,
        avatar -> Text,
    }
}

table! {
    blacklist (id) {
        id -> Int8,
        reason -> Text,
        author -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    config (id) {
        id -> Int8,
        prefix -> Text,
        max_queue -> Int4,
        no_duplicate -> Bool,
        keep_alive -> Bool,
        guild_roles -> Array<Int8>,
        playlist_roles -> Array<Int8>,
        player_roles -> Array<Int8>,
        queue_roles -> Array<Int8>,
        track_roles -> Array<Int8>,
        playing_log -> Int8,
        player_log -> Int8,
        queue_log -> Int8,
    }
}

table! {
    guild (id) {
        id -> Int8,
        name -> Text,
        icon -> Text,
        owner -> Int8,
        member_count -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    guild_log (id) {
        id -> Int8,
        guild -> Int8,
        action -> Text,
        author -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    guild_stat (id) {
        id -> Int8,
        guild -> Int8,
        author -> Int8,
        title -> Text,
        created_at -> Timestamp,
    }
}

table! {
    playlist (id) {
        id -> Int8,
        guild -> Int8,
        name -> Text,
        author -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    playlist_item (id) {
        id -> Int8,
        playlist -> Int8,
        track -> Text,
        title -> Text,
        uri -> Text,
        length -> Int4,
    }
}

joinable!(playlist_item -> playlist (playlist));

allow_tables_to_appear_in_same_query!(
    account,
    blacklist,
    config,
    guild,
    guild_log,
    guild_stat,
    playlist,
    playlist_item,
);
