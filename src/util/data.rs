/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use pickledb::{PickleDb, PickleDbDumpPolicy};
use rusqlite::Connection;

pub fn get_pickle_database(guild_id: u64, db_name: String) -> PickleDb {
    let path = construct_data_path(guild_id, db_name);

    let db = match PickleDb::load_yaml(&path, PickleDbDumpPolicy::AutoDump) {
        Ok(d) => d,
        Error => PickleDb::new_yaml(&path, PickleDbDumpPolicy::AutoDump),
    };

    db

}

fn construct_data_path(guild_id: u64, db_name: String) -> String {
    let mut path = String::from("./data/");
    path.push_str(&guild_id.to_string());
    path.push_str(&"/");
    path.push_str(&db_name);

    path
}

pub fn get_sqlite_database(guild_id: u64, db_name: String) -> Connection {
    let mut conn = Connection::open(construct_data_path(guild_id, db_name)).unwrap();

    conn
}