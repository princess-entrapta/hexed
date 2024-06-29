CREATE TABLE game (
    id INTEGER PRIMARY KEY,
    game_status TEXT,
    user_owner TEXT,
    scenario_id INTEGER
);

CREATE TABLE scenario (
    id INTEGER PRIMARY KEY,
    map_id INTEGER,
    scenario_description TEXT
);

CREATE TABLE scenario_player(
    id INTEGER PRIMARY KEY,
    scenario_id INTEGER,
    player_points INTEGER
);

CREATE TABLE available_class(
    scenario_player_id INTEGER,
    player_points INTEGER,
    game_class TEXT
);

CREATE TABLE seated_player(
    id INTEGER PRIMARY KEY,
    user_id TEXT,
    game_id INTEGER,
    scenario_player_id INTEGER
);

CREATE TABLE drop_zone_tile(
    scenario_player_id INTEGER,
    x INTEGER,
    y INTEGER
);

CREATE TABLE entity (
    id INTEGER PRIMARY KEY,
    game_id INTEGER,
    seat_id INTEGER,
    x INTEGER,
    y INTEGER,
    last_move_time INTEGER,
    next_move_time INTEGER,
    game_class TEXT
);

CREATE TABLE action_log(
    game_id INTEGER,
    turn_time INTEGER,
    action_name TEXT,
    caster INTEGER,
    target_entity INTEGER
);

CREATE TABLE tile (
    map_id INTEGER,
    x INTEGER,
    y INTEGER,
    tile_type TEXT
);

CREATE TABLE entity_resource (
    entity_id INTEGER,
    game_id INTEGER,
    resource_name TEXT,
    resource_max FLOAT,
    resource_current FLOAT,
    resource_per_turn FLOAT
);

CREATE TABLE user (
    username TEXT NOT NULL,
    passhash TEXT NOT NULL,
    roles TEXT
);

INSERT INTO
    scenario (id, map_id, scenario_description)
VALUES
    (1, 1, "Test data"),
    (2, 2, "Simple arena");

INSERT INTO
    scenario_player (
        id,
        scenario_id,
        player_points
    )
VALUES
    (1, 1, 100),
    (2, 1, 100),
    (3, 2, 100),
    (4, 2, 100);

INSERT INTO
    drop_zone_tile (scenario_player_id, x, y)
VALUES
    (1, -2, 0),
    (2, 2, 0),
    (3, -4, 0),
    (3, -3, -1),
    (3, -3, 1),
    (3, -2, 0),
    (4, 4, 0),
    (4, 3, -1),
    (4, 3, 1),
    (4, 2, 0);

INSERT INTO
    available_class (scenario_player_id, game_class, player_points)
VALUES
    (1, "Warrior", 25),
    (2, "Warrior", 25),
    (3, "Warrior", 25),
    (3, "Archer", 25),
    (4, "Warrior", 25),
    (4, "Archer", 25);

INSERT INTO
    seated_player (id, user_id, game_id, scenario_player_id)
VALUES
    (1, "diane", 1, 1),
    (2, "arthur", 1, 2);

INSERT INTO
    tile (map_id, x, y, tile_type)
VALUES
    (1, -2, 0, 'Floor'),
    (1, 2, 0, 'Floor'),
    (1, -1, 1, 'Floor'),
    (1, 1, 1, 'Floor'),
    (1, -1, -1, 'Floor'),
    (1, 1, -1, 'Floor'),
    (1, 0, 0, 'Wall'),
    (2, -4, 0, 'Floor'),
    (2, -2, 0, 'Floor'),
    (2, -0, 0, 'Floor'),
    (2, 2, 0, 'Floor'),
    (2, 4, 0, 'Floor'),
    (2, -3, 1, 'Floor'),
    (2, -1, 1, 'Floor'),
    (2, 1, 1, 'Floor'),
    (2, 3, 1, 'Floor'),
    (2, -3, -1, 'Floor'),
    (2, -1, -1, 'Floor'),
    (2, 1, -1, 'Floor'),
    (2, 3, -1, 'Floor'),
    (2, -2, -2, 'Floor'),
    (2, 0, -2, 'Floor'),
    (2, 2, -2, 'Floor'),
    (2, -2, 2, 'Floor'),
    (2, 0, 2, 'Floor'),
    (2, 2, 2, 'Floor'),
    (2, -1, 3, 'Floor'),
    (2, 1, 3, 'Floor'),
    (2, -1, -3, 'Floor'),
    (2, 1, -3, 'Floor');

INSERT INTO
    game (id, scenario_id, game_status, user_owner)
VALUES
    (1, 1, 'Running', 'diane'),
    (2, 2, 'Open', 'diane');

INSERT INTO
    entity_resource (
        entity_id,
        game_id,
        resource_name,
        resource_max,
        resource_current,
        resource_per_turn
    )
VALUES
    (1, 1, 'hp', 100, 100, 0.1),
    (2, 1, 'hp', 100, 100, 0.1);

INSERT INTO
    entity (
        id,
        game_id,
        scenario_player_id,
        seat_id,
        x,
        y,
        last_move_time,
        next_move_time,
        game_class
    )
VALUES
    (
        1,
        1,
        1,
        1,
        -2,
        0,
        0,
        0,
        'Warrior'
    ),
    (
        2,
        1,
        2,
        2,
        2,
        0,
        0,
        0,
        'Warrior'
    );