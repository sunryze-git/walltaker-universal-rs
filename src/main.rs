use std::{fs, path::PathBuf};
use std::path::Path;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio::time::{sleep, Duration};
use wallpaper;

mod api;
use api::fetch_walltaker_link;

// I just wanna note something important here.
// I was losing my mind trying to figure out how to make this.
// I started in C# originally and encountered UI problems there.
// I then moved to C++, and encountered SOOOO MUCH BOILERPLATE.
// I then tried this and I got it working in 30 minutes.
// What the hell.

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    link_id: String,
    api_key: String,
}

#[derive(Deserialize)]
struct WalltakerLink {
    post_url: String,
}

async fn load_config(path: &str) -> anyhow::Result<ConfigFile> {
    // Doesn't exist
    if !Path::exists(Path::new(path)) {
        // create a default and exit
        let default_config = ConfigFile {
            link_id: "your_link_id".to_string(),
            api_key: "your_api_key".to_string(),
        };
        let default_data = serde_json::to_string_pretty(&default_config)?;
        fs::write(path, default_data)?;
        eprintln!("Config file not found. Created default config at: {}", path);
        std::process::exit(1);
    }
    let data = fs::read_to_string(path)?;
    
    // Attempt to parse the config file
    if data.trim().is_empty() {
        eprintln!("Config file is empty. Please provide valid configuration.");
        std::process::exit(1);
    }

    // Try to deserialize the config file
    let conf: ConfigFile = serde_json::from_str(&data).map_err(|e| {
        eprintln!("Failed to parse config file: {}", e);
        std::process::exit(1);
    })?;
    
    // Validate required fields. shouldnt be empty, link_id should be numeric
    if conf.link_id.is_empty() || conf.api_key.is_empty() {
        eprintln!("Config file is missing required fields. Please provide valid configuration.");
        std::process::exit(1);
    }
    if conf.link_id.chars().any(|c| !c.is_numeric()) {
        eprintln!("Config file link_id should be numeric. Please provide valid configuration.");
        std::process::exit(1);
    }
    Ok(conf)
}

async fn download_file(url: &str, dest: &PathBuf) -> anyhow::Result<()> {
    let resp = Client::new().get(url).send().await?;
    let bytes = resp.bytes().await?;
    fs::write(dest, &bytes)?;
    Ok(())
}

fn set_wallpaper(path: &str) -> anyhow::Result<()> {
    wallpaper::set_from_path(path).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf = load_config("./config.json").await?;
    println!("Config loaded: UserId = {}, API Key = {}", conf.link_id, conf.api_key);

    let wallpaper_path = std::env::temp_dir().join("walltaker_wallpaper.jpg");
    let api_url = format!("https://walltaker.joi.how/api/links/{}.json", conf.link_id);
    let mut last_wallpaper_url = String::new();

    loop {
        let link = match fetch_walltaker_link(&Client::new(), &api_url).await {
            Ok(link) => link,
            Err(e) => {
                eprintln!("Failed to fetch wallpaper information: {}. Retrying...", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        let new_wallpaper_url = link.post_url;
        if new_wallpaper_url.is_empty() || new_wallpaper_url == last_wallpaper_url {
            sleep(Duration::from_secs(10)).await;
            continue;
        }
        println!("New wallpaper URL found: {}", new_wallpaper_url);
        last_wallpaper_url = new_wallpaper_url.clone();
        println!("Downloading new wallpaper to: {:?}", wallpaper_path);
        if let Err(e) = download_file(&new_wallpaper_url, &wallpaper_path).await {
            eprintln!("Failed to download wallpaper: {}. Retrying...", e);
            sleep(Duration::from_secs(10)).await;
            continue;
        }
        println!("Wallpaper downloaded successfully! Waiting for next check...");
        set_wallpaper(&wallpaper_path.to_string_lossy())?;
        sleep(Duration::from_secs(10)).await;
    }
}
