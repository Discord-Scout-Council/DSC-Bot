/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use super::data::get_global_pickle_database;

pub fn contains_banned_word(content: &String) -> bool{
    let mut db = get_global_pickle_database("banned_words.db");
    let mut banned_words = db.get_all();

    let mut lower_content = content.to_lowercase();

    for w in banned_words.iter() {
        if lower_content.contains(w) {
            return true;
        } else {
            continue;
        }
    }
    return false;
}