use crate::models::Message;
use crate::schema::{messages, summary_probs};
use crate::utils::cosine_similarity;
use diesel::query_dsl::methods::*;
use diesel::{Connection, PgConnection};
use diesel::{ExpressionMethods, RunQueryDsl};
use dotenvy::dotenv;
use kdam::{tqdm, BarExt};
use std::collections::HashMap;
use std::env;

pub fn connect_db() -> PgConnection {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let conn = PgConnection::establish(&db_url).unwrap();
    println!("Connected to database!");
    conn
}

pub fn insert_messages_to_db(conn: &mut PgConnection, messages: &HashMap<String, Vec<String>>) {
    let values = messages
        .iter()
        .flat_map(|(filename, messages)| {
            messages.iter().map(move |message| {
                (
                    messages::filename.eq(filename),
                    messages::message.eq(message),
                )
            })
        })
        .collect::<Vec<_>>();

    let mut progress_bar = tqdm!(desc = "Inserting messages to db", total = values.len());

    for batch in values.chunks(10_000) {
        diesel::insert_into(messages::table)
            .values(batch)
            .execute(conn)
            .unwrap();

        progress_bar.update(batch.len());
    }
    eprintln!();
}

// read inserted messages from db and calculate their summary probabilities by using fasttext
// sentence vectors and use cosine similarity to calculate the probability of each sentence
// being the summary of the message, repeat this for each message group corresponding to a filename
pub fn insert_summary_probs_to_db(conn: &mut PgConnection, ft: &fasttext::FastText, lang: &str) {
    let filenames = messages::table
        .select(messages::filename)
        .distinct()
        .load::<String>(conn)
        .unwrap();

    for filename in tqdm!(
        filenames.iter(),
        desc = "Inserting summary probs to db",
        total = filenames.len(),
        position = 0
    ) {
        let messages = messages::table
            .filter(messages::filename.eq(filename))
            .load::<Message>(conn)
            .unwrap();

        let message_vectors: Vec<Vec<f32>> = messages
            .iter()
            .map(|message| ft.get_sentence_vector(&message.message).unwrap())
            .collect();

        let summary = summary_message_vectors(&message_vectors);

        let values = tqdm!(
            messages.iter().enumerate(),
            desc = "Calculating summary probs",
            position = 1
        )
        .map(|(i, message)| {
            let prob = cosine_similarity(&summary, &message_vectors[i]);
            (
                summary_probs::message_id.eq(message.id),
                summary_probs::lang.eq(lang),
                summary_probs::prob.eq(prob),
            )
        })
        .collect::<Vec<_>>();

        let batch_size = 10_000;
        for batch in tqdm!(
            values.chunks(batch_size),
            desc = "Inserting summary probs",
            total = values.len() / batch_size,
            position = 2
        ) {
            diesel::insert_into(summary_probs::table)
                .values(batch)
                .execute(conn)
                .unwrap();
        }
    }
}

#[allow(dead_code)]
fn summary_probability(message_vectors: &Vec<Vec<f32>>, i: usize) -> f32 {
    let mut prob = 0.0;
    for (j, message_vector) in message_vectors.iter().enumerate() {
        if i == j {
            continue;
        }
        let similarity = cosine_similarity(&message_vector, &*message_vectors[i]);
        prob += similarity;
    }
    prob /= message_vectors.len() as f32 - 1.0;
    prob
}

fn summary_message_vectors(message_vectors: &Vec<Vec<f32>>) -> Vec<f32> {
    let mut summary_vector = vec![0.0; message_vectors[0].len()];
    for message_vector in message_vectors {
        for (i, value) in message_vector.iter().enumerate() {
            summary_vector[i] += value;
        }
    }
    for value in summary_vector.iter_mut() {
        *value /= message_vectors.len() as f32;
    }
    summary_vector
}
