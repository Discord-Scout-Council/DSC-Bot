/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use pickledb::{PickleDb, PickleDbDumpPolicy};
use rusqlite::{params, Connection};
use std::fs::create_dir_all;

pub fn get_pickle_database(guild_id: &u64, db_name: &str) -> PickleDb {
    let path = construct_data_path(&guild_id, &db_name);

    let db = match PickleDb::load_yaml(&path, PickleDbDumpPolicy::AutoDump) {
        Ok(d) => d,
        Error => PickleDb::new_yaml(&path, PickleDbDumpPolicy::AutoDump),
    };

    db
}

pub fn get_global_pickle_database(db_name: &str) -> PickleDb {
    let mut path: String = String::from("./data/");
    path.push_str(db_name);
    let db = match PickleDb::load_yaml(&path, PickleDbDumpPolicy::AutoDump) {
        Ok(d) => d,
        Error => PickleDb::new_yaml(&path, PickleDbDumpPolicy::AutoDump),
    };

    db
}

fn construct_data_path(guild_id: &u64, db_name: &str) -> String {
    let mut path = String::from("./data/");
    path.push_str(&guild_id.to_string());
    path.push_str(&"/");
    create_directories(&path);
    path.push_str(&db_name);

    path
}

fn create_directories(path: &String) {
    create_dir_all(path).unwrap();
}

fn get_sqlite_database(guild_id: &u64, db_name: &str) -> Connection {
    let mut conn = Connection::open(construct_data_path(&guild_id, &db_name)).unwrap();

    conn
}

pub fn get_strike_database(guild_id: &u64) -> Connection {
    let conn = get_sqlite_database(guild_id, &"strikes.db");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS strikes (
                                id INTEGER PRIMARY KEY,
                                userid TEXT NOT NULL,
                                reason TEXT)",
        params![],
    )
    .unwrap();

    conn
}

pub fn init_guild_settings(db: &mut PickleDb) {
    //* Question of the Day
    db.set("qotd_role", &0u64);
    db.set("qotd_channel", &0u64);
    db.set("qotd_suggest_channel", &0u64);
    
    db.set("modlogs_channel", &0u64);
}