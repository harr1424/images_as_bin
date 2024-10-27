use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest::blocking;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    sync::Mutex,
};

#[derive(Debug, Deserialize)]
struct Post {
    title: String,
    images: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if Path::new("images.bin").exists() {
        println!("File 'images.bin' exists. Loading saved images...");
        inspect_saved_images("images.bin")?;

        return Ok(());
    }
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
    bar.finish_with_message("Image downloads complete!");

    // Serialize and save image data using bincode
    save_image_data(&image_data.lock().unwrap(), "images.bin")?;

    Ok(())
}

fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = blocking::get(url)?;
    Ok(response.bytes()?.to_vec())
}

fn save_image_data(image_data: &HashMap<String, Vec<u8>>, path: &str) -> io::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(image_data).expect("Serialization failed");
    let mut file = File::create(path)?;
    file.write_all(&encoded)?;
    Ok(())
}

fn load_image_data(path: &str) -> io::Result<HashMap<String, Vec<u8>>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let decoded: HashMap<String, Vec<u8>> =
        bincode::deserialize(&buffer).expect("Deserialization failed");
    Ok(decoded)
}

fn inspect_saved_images(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let image_data: HashMap<String, Vec<u8>> = load_image_data(path)?;

    if image_data.is_empty() {
        println!("No images found in the file.");
        return Ok(());
    }

    for (url, data) in image_data.iter().take(1) {
        println!("URL: {}", url);
        println!("Image size (bytes): {}", data.len());
        println!(
            "Image data sample: {:?}",
            &data[..std::cmp::min(10, data.len())]
        );
    }

    Ok(())
}
