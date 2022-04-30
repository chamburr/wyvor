CREATE TABLE account
(
    id          bigint    NOT NULL PRIMARY KEY,
    email       text      NOT NULL UNIQUE,
    username    text      NOT NULL UNIQUE,
    description text      NOT NULL,
    password    text      NOT NULL,
    status      integer   NOT NULL,
    created_at  timestamp NOT NULL
);

CREATE TABLE space
(
    id          bigint    NOT NULL PRIMARY KEY,
    name        text      NOT NULL,
    description text      NOT NULL,
    public      boolean   NOT NULL,
    created_at  timestamp NOT NULL
);

CREATE TABLE member
(
    space      bigint    NOT NULL REFERENCES space (id) ON DELETE CASCADE,
    account    bigint    NOT NULL REFERENCES account (id),
    role       integer   NOT NULL,
    created_at timestamp NOT NULL,
    PRIMARY KEY (space, account)
);

CREATE TABLE playlist
(
    id         bigint    NOT NULL PRIMARY KEY,
    space      bigint    NOT NULL REFERENCES space (id) ON DELETE CASCADE,
    name       text      NOT NULL,
    items      integer[] NOT NULL,
    created_at timestamp NOT NULL
);
