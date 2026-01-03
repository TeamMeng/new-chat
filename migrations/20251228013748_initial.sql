-- Add migration script here

-- create user table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    ws_id BIGINT NOT NULL,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL UNIQUE,
    -- hashed argon2 password, length 97
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces(
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- add super user 0 and workspace 0
BEGIN;
INSERT INTO users(id, ws_id, fullname, email, password_hash)
  VALUES (0, 0, 'super user', 'super@none.org', '');

INSERT INTO workspaces(id, name, owner_id)
  VALUES (0, 'none', 0);
COMMIT;

-- alter user table to make ws_id not null
ALTER TABLE users
  ALTER COLUMN ws_id SET NOT NULL;

-- add foreign key constraint for ws_id for users
ALTER TABLE users
  ADD CONSTRAINT users_ws_id_fk FOREIGN KEY (ws_id) REFERENCES workspaces(id);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

CREATE TYPE chat_type AS ENUM(
    'single',
    'group',
    'private_channel',
    'public_channel'
);

-- create chat table
CREATE TABLE IF NOT EXISTS chats(
    id BIGSERIAL PRIMARY KEY,
    ws_id BIGINT NOT NULL REFERENCES workspaces(id),
    name VARCHAR(64),
    TYPE chat_type NOT NULL,
    -- user id list
    members bigint[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (ws_id, name)
);

-- create message table
CREATE TABLE IF NOT EXISTS messages(
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content text NOT NULL,
    files text[] DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id by created_at desc
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);

-- create index for chat members
CREATE INDEX IF NOT EXISTS chat_members_index ON chats USING GIN(members);
