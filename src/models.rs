use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Dban {
    pub id: i32,
    pub reason: String,
    pub guild_id: String,
    pub userid: String,
    pub is_withdrawn: bool,
}

#[derive(Deserialize)]
pub struct StrikeLog {
  pub id: i32,
  pub moderator: String,
  pub reason: String,
}

#[derive(Deserialize)]
pub struct GetStrike {
  pub userid: String,
  pub moderator: String,
  pub reason: String,
  pub is_withdrawn: bool,
}

#[derive(Deserialize, Default)]
pub struct Badge {
  pub badge: String,
}

#[derive(Deserialize)]
pub struct DbanList {
  pub id: i32,
  pub reason: String,
  pub is_withdrawn: bool,
}
