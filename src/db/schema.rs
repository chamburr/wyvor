table! {
    account (id) {
        id -> Int8,
        email -> Text,
        username -> Text,
        description -> Text,
        password -> Text,
        status -> Int4,
        created_at -> Timestamp,
    }
}

table! {
    member (space, account) {
        space -> Int8,
        account -> Int8,
        role -> Int4,
        created_at -> Timestamp,
    }
}

table! {
    playlist (id) {
        id -> Int8,
        space -> Int8,
        name -> Text,
        items -> Array<Int4>,
        created_at -> Timestamp,
    }
}

table! {
    space (id) {
        id -> Int8,
        name -> Text,
        description -> Text,
        public -> Bool,
        created_at -> Timestamp,
    }
}

joinable!(member -> account (account));
joinable!(member -> space (space));
joinable!(playlist -> space (space));

allow_tables_to_appear_in_same_query!(account, member, playlist, space,);
