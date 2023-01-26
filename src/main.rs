use faster_all::config::read_config;
use faster_all::database::{connect_db, insert_messages_to_db, insert_summary_probs_to_db};
use faster_all::utils::{download_model, load_model, log_lang, read_csv_files};

#[tokio::main]
async fn main() {
    let conn = &mut connect_db();
    let cfg = read_config();

    let file_messages = read_csv_files();
    insert_messages_to_db(conn, &file_messages);

    for lang in cfg.langs {
        let lang = lang.as_str();

        log_lang(lang);
        let filename = download_model(lang).await.unwrap();
        let ft = load_model(&filename);
        insert_summary_probs_to_db(conn, &ft, lang);
    }
}
