-- insert workspaces
INSERT INTO workspaces(name, owner_id)
    VALUES ('acme', 0), ('foo', 0), ('bar', 0);

-- insert users
INSERT INTO
    users(ws_id, fullname, email, password_hash)
VALUES
(1, 'TeamTest', 'Test@123.com', '$argon2id$v=19$m=19456,t=2,p=1$MxGhY+ib/kplwBPLa7u2ug$c5h9u7Sc8Px8J5+qgNdOjSY7ZJO2QN4rugKpapGW4XU'),
(1, 'Alice Test', 'Alice@123.com', '$argon2id$v=19$m=19456,t=2,p=1$MxGhY+ib/kplwBPLa7u2ug$c5h9u7Sc8Px8J5+qgNdOjSY7ZJO2QN4rugKpapGW4XU'),
(1, 'Bob Test', 'Bob@123.com', '$argon2id$v=19$m=19456,t=2,p=1$MxGhY+ib/kplwBPLa7u2ug$c5h9u7Sc8Px8J5+qgNdOjSY7ZJO2QN4rugKpapGW4XU'),
(1, 'Charlie Test', 'Charlie@123.com', '$argon2id$v=19$m=19456,t=2,p=1$MxGhY+ib/kplwBPLa7u2ug$c5h9u7Sc8Px8J5+qgNdOjSY7ZJO2QN4rugKpapGW4XU'),
(1, 'Daisy Test', 'Daisy@123.com', '$argon2id$v=19$m=19456,t=2,p=1$MxGhY+ib/kplwBPLa7u2ug$c5h9u7Sc8Px8J5+qgNdOjSY7ZJO2QN4rugKpapGW4XU');

-- insert 4 chats
INSERT INTO chats (ws_id, name, type, members)
    VALUES (1, 'general', 'public_channel', '{1, 2, 3, 4, 5}'),
(1, 'private', 'private_channel', '{1, 2, 3}');

-- insert unnamed chat
INSERT INTO chats(ws_id, type, members)
    VALUES (1, 'single', '{1, 2}'),
(1, 'group', '{1, 3, 4}');
