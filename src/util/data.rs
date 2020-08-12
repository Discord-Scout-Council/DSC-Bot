/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use pickledb::{PickleDb, PickleDbDumpPolicy};
use sqlx::postgres::PgPool;
use std::env;
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

pub fn init_guild_settings(db: &mut PickleDb) {
    //* Question of the Day
    db.set("modlogs_channel", &0u64);
}

pub async fn obtain_pg_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let url = match env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(err) => {
            return Err(Box::new(err));
        }
    };

    //Connect and return a pool
    let pool = PgPool::builder().max_size(5).build(&url).await?;

    Ok(pool)
}
