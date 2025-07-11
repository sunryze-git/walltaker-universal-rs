use reqwest::Client;
use crate::WalltakerLink;
use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct PostRequest {
    pub api_key: String,
    pub r#type: String,
    pub text: String,
}

pub async fn fetch_walltaker_link(client: &Client, api_url: &str) -> Result<WalltakerLink> {
    let resp = client.get(api_url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch wallpaper information");
    }
    let link: WalltakerLink = resp.json().await?;
    Ok(link)
}

pub async fn send_walltaker_opinion(client: &Client, post_url: &str, api_key: &str, response_type: &str, response_text: &str) -> Result<()> {
    let post_request = PostRequest {
        api_key: api_key.to_string(),
        r#type: response_type.to_lowercase(),
        text: response_text.to_string(),
    };
    let resp = client.post(post_url)
        .header("Accept", "application/json")
        .json(&post_request)
        .send()
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("Failed to send opinion: {}", resp.status());
    }
    Ok(())
}
