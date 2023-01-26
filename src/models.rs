use diesel::prelude::*;

#[derive(Queryable, Debug)]
pub struct Message {
    pub id: i32,
    pub filename: String,
    pub message: String,
}

#[derive(Queryable, Debug)]
pub struct SummaryProb {
    pub id: i32,
    pub message_id: i32,
    pub lang: String,
    pub prob: f32,
}
