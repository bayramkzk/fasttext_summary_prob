use flate2::read::GzDecoder;
use kdam::{tqdm, BarExt};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub fn log_lang(lang: &str) {
    let status = format!(" Processing language: {} ", lang.to_uppercase());
    println!("\n{:-^80}", status);
}

pub async fn download_model(lang: &str) -> anyhow::Result<String> {
    let folder_path = Path::new("models");
    if !folder_path.exists() {
        fs::create_dir(folder_path)?;
    }

    let bin_path = folder_path.join(format!("cc.{}.300.bin", lang));
    let bin_fullname = bin_path.to_str().unwrap().to_string();
    let gzip_path = folder_path.join(format!("cc.{}.300.bin.gz", lang));

    if bin_path.exists() {
        println!("Skipping download for model {}", bin_fullname);
        return Ok(bin_fullname);
    }

    if !gzip_path.exists() {
        download_gzip_file(lang, gzip_path.as_path()).await?;
    }

    extract_gzip_file(gzip_path.as_path(), bin_path.as_path())?;

    fs::remove_file(gzip_path)?;

    Ok(bin_fullname)
}

fn extract_gzip_file(gzip_path: &Path, bin_path: &Path) -> Result<(), anyhow::Error> {
    let gzip_file = File::open(gzip_path)?;
    let gzip_size = gzip_file.metadata()?.len() as usize;

    let gzip_reader = BufReader::new(gzip_file);
    let mut gzip_decoder = GzDecoder::new(gzip_reader);
    let mut bin_file = File::create(bin_path)?;

    let mut progress_bar = tqdm!(desc = "Decompressing model", total = gzip_size);
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = gzip_decoder.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        bin_file.write_all(&buffer[..bytes_read])?;
        progress_bar.update(bytes_read);
    }
    eprintln!();
    Ok(())
}

async fn download_gzip_file(lang: &str, gzip_path: &Path) -> Result<File, anyhow::Error> {
    let base_url = "https://dl.fbaipublicfiles.com/fasttext/vectors-crawl";
    let url = format!("{}/cc.{}.300.bin.gz", base_url, lang);
    let mut res = reqwest::get(url).await?;
    let content_length = res.content_length().unwrap_or(0) as usize;
    let mut gzip_file = File::create(gzip_path)?;
    let mut progress_bar = tqdm!(desc = "Downloading model", total = content_length);
    while let Some(chunk) = res.chunk().await? {
        gzip_file.write_all(&chunk)?;
        progress_bar.update(chunk.len());
    }
    eprintln!();
    Ok(gzip_file)
}

pub fn load_model(filename: &str) -> fasttext::FastText {
    let mut ft = fasttext::FastText::new();
    eprint!("Loading model");
    ft.load_model(&filename).unwrap();
    eprintln!();
    ft
}

// read csv files from data and get "Message" column of all of them
// return a hashmap of (filename, messages) pairs
pub fn read_csv_files() -> HashMap<String, Vec<String>> {
    let mut messages = HashMap::new();
    let paths = fs::read_dir("data").unwrap();
    let paths = paths.filter_map(|x| x.ok());
    let paths = paths.collect::<Vec<_>>();

    for path in tqdm!(
        paths.iter(),
        desc = "Reading csv files",
        total = paths.len()
    ) {
        let path = path.path();
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        let file = File::open(path).unwrap();
        let mut reader = csv::Reader::from_reader(file);
        let headers = reader.headers().unwrap();
        let message_index = headers.iter().position(|x| x == "Message").unwrap();
        let mut messages_in_file = Vec::new();
        for record in reader.records() {
            let record = record.unwrap();
            let message = record.get(message_index).unwrap().to_string();
            if !message.is_empty() {
                messages_in_file.push(message);
            }
        }
        messages.insert(filename, messages_in_file);
    }
    messages
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut a_norm = 0.0;
    let mut b_norm = 0.0;
    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        a_norm += a[i] * a[i];
        b_norm += b[i] * b[i];
    }
    dot_product / (a_norm.sqrt() * b_norm.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_model() {
        let model = download_model("tr").await;
        assert!(model.is_ok());
        let model = model.unwrap();

        let mut ft = fasttext::FastText::new();
        let res = ft.load_model(&model);
        assert!(res.is_ok());
    }

    #[test]
    fn test_read_csv_files() {
        let messages = read_csv_files();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages["test1.csv"].len(), 3);
        assert_eq!(messages["test2.csv"].len(), 2);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let c = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.9746318);
        assert_eq!(cosine_similarity(&a, &c), 1.0);
    }
}
