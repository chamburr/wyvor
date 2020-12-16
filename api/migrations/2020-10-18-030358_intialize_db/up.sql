CREATE FUNCTION pseudo_encrypt(VALUE bigint) RETURNS bigint AS
$$
DECLARE
    l1 bigint;
    l2 bigint;
    r1 bigint;
    r2 bigint;
    i  int := 0;
BEGIN
    l1 := (VALUE >> 32) & 4294967295::bigint;
    r1 := VALUE & 4294967295;
    WHILE i < 3
        LOOP
            l2 := r1;
            r2 := l1 # ((((1366.0 * r1 + 150889) % 714025) / 714025.0) * 32767 * 32767)::int;
            l1 := l2;
            r1 := r2;
            i := i + 1;
        END LOOP;
    RETURN ((l1::bigint << 32) + r1);
END;
$$ LANGUAGE plpgsql strict
                    immutable;

CREATE FUNCTION update_timestamp() RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql strict
                    immutable;

CREATE SEQUENCE seq_serial;

CREATE TABLE account
(
    id            bigint  NOT NULL PRIMARY KEY,
    username      text    NOT NULL,
    discriminator integer NOT NULL,
    avatar        text    NOT NULL
);

CREATE TABLE blacklist
(
    id         bigint    NOT NULL PRIMARY KEY,
    reason     text      NOT NULL,
    author     bigint    NOT NULL,
    created_at timestamp NOT NULL DEFAULT current_timestamp
);

CREATE TABLE config
(
    id             bigint   NOT NULL PRIMARY KEY,
    prefix         text     NOT NULL DEFAULT '.',
    max_queue      integer  NOT NULL DEFAULT 1000,
    no_duplicate   boolean  NOT NULL DEFAULT FALSE,
    keep_alive     boolean  NOT NULL DEFAULT FALSE,
    guild_roles    bigint[] NOT NULL DEFAULT '{}',
    playlist_roles bigint[] NOT NULL DEFAULT '{}',
    player_roles   bigint[] NOT NULL DEFAULT '{}',
    queue_roles    bigint[] NOT NULL DEFAULT '{}',
    track_roles    bigint[] NOT NULL DEFAULT '{}',
    playing_log    bigint   NOT NULL DEFAULT 0,
    player_log     bigint   NOT NULL DEFAULT 0,
    queue_log      bigint   NOT NULL DEFAULT 0
);

CREATE TABLE guild
(
    id           bigint    NOT NULL PRIMARY KEY,
    name         text      NOT NULL,
    icon         text      NOT NULL,
    owner        bigint    NOT NULL,
    member_count integer   NOT NULL,
    created_at   timestamp NOT NULL DEFAULT current_timestamp,
    updated_at   timestamp NOT NULL DEFAULT current_timestamp
);

CREATE TABLE guild_log
(
    id         bigint    NOT NULL PRIMARY KEY DEFAULT pseudo_encrypt(nextval('seq_serial')),
    guild      bigint    NOT NULL,
    action     text      NOT NULL,
    author     bigint    NOT NULL,
    created_at timestamp NOT NULL DEFAULT current_timestamp
);

CREATE TABLE guild_stat
(
    id         bigint    NOT NULL PRIMARY KEY DEFAULT pseudo_encrypt(nextval('seq_serial')),
    guild      bigint    NOT NULL,
    author     bigint    NOT NULL,
    title      text      NOT NULL,
    created_at timestamp NOT NULL DEFAULT current_timestamp
);

CREATE TABLE playlist
(
    id         bigint    NOT NULL PRIMARY KEY DEFAULT pseudo_encrypt(nextval('seq_serial')),
    guild      bigint    NOT NULL,
    name       text      NOT NULL,
    author     bigint    NOT NULL,
    created_at timestamp NOT NULL DEFAULT current_timestamp
);

CREATE TABLE playlist_item
(
    id       bigint  NOT NULL PRIMARY KEY DEFAULT pseudo_encrypt(nextval('seq_serial')),
    playlist bigint  NOT NULL REFERENCES playlist (id),
    track    text    NOT NULL,
    title    text    NOT NULL,
    uri      text    NOT NULL,
    length   integer NOT NULL
);

CREATE TRIGGER t_guild_update
    BEFORE UPDATE
    ON guild
    FOR EACH ROW
EXECUTE PROCEDURE update_timestamp();
