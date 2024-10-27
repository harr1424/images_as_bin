use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest::blocking;
use serde::Deserialize;
use std::{collections::HashMap, fs::File, sync::Mutex};

#[derive(Debug, Deserialize)]
struct Post {
    title: String,
    images: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let posts: Vec<Post> = serde_json::from_reader(File::open("backup.json")?)?;
    let image_data: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());

    let bar = ProgressBar::new(posts.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
            )?
            .progress_chars("#>-"),
    );
    posts.par_iter().for_each(|post| {
        bar.set_message(format!("Downloading image for: {}", post.title));
        for image_url in &post.images {
            match download_image(image_url) {
                Ok(data) => {
                    let mut data_lock = image_data.lock().unwrap();
                    data_lock.insert(image_url.clone(), data);
                                }
                Err(e) => {
                    eprintln!("Failed to download {}: {}", image_url, e);
                }
            }
        }
        bar.inc(1);
    });
    bar.finish_with_message("PDF generation complete!");
    serde_json::to_writer(File::create("images.json")?, &image_data)?;

    Ok(())
}

fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = blocking::get(url)?;
    Ok(response.bytes()?.to_vec())
}
