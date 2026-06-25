use regex::Regex;
use reqwest::header::REFERER;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

pub trait Parser {
    fn audio(&self) -> &str;
    fn title(&self) -> &str;
    fn extension(&self) -> &str;

    fn headers(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub struct SoundgasmParser {
    audio: String,
    title: String,
    extension: String,
}

impl SoundgasmParser {
    const AUDIO_REGEX: &str = r#"(https:\/\/media\.soundgasm\.net\/sounds\/[^\r\n\t\f\v"]+)"#;
    const TITLE_SELECTOR: &str = "div.jp-title";

    pub async fn new(url: &str) -> Result<Self, String> {
        let (audio, title, extension) = Self::parse_info(url).await?;
        Ok(Self {
            audio,
            title,
            extension,
        })
    }

    async fn parse_info(url: &str) -> Result<(String, String, String), String> {
        info_from_page(url, Self::AUDIO_REGEX, Self::TITLE_SELECTOR).await
    }
}

impl Parser for SoundgasmParser {
    fn audio(&self) -> &str {
        &self.audio
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn extension(&self) -> &str {
        &self.extension
    }
}

pub struct VocarooParser {
    audio: String,
    title: String,
    extension: String,
}

impl VocarooParser {
    pub fn new(id: &str) -> Self {
        let audio = format!("https://media1.vocaroo.com/mp3{id}");
        let title = format!("Vocaroo {id}");
        let extension = "mp3".to_owned();
        Self {
            audio,
            title,
            extension,
        }
    }
}

impl Parser for VocarooParser {
    fn audio(&self) -> &str {
        &self.audio
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn extension(&self) -> &str {
        &self.extension
    }

    fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(REFERER.to_string(), "https://vocaroo.com/".to_owned());
        headers
    }
}

pub struct AudiochanParser {
    audio: String,
    title: String,
    extension: String,
}

impl AudiochanParser {
    pub async fn new(url: &str) -> Result<Self, String> {
        let (audio, title, extension) = Self::parse_info(url).await?;
        Ok(Self {
            audio,
            title,
            extension,
        })
    }

    async fn parse_info(url: &str) -> Result<(String, String, String), String> {
        let base_url = "audiochan.com";
        let slug = url
            .split('/')
            .next_back()
            .ok_or(format!("Failed to parse slug from URL: {}", url))?;
        let body = reqwest::get(format!("https://api.{base_url}/audios/slug/{slug}"))
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        let json: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let title = json["title"]
            .as_str()
            .ok_or("Title is missing from API response")?
            .to_owned();
        let audio_file = &json["audioFile"];
        let audio_key = audio_file["key"]
            .as_str()
            .ok_or("Audio key is missing from API response")?
            .to_owned();
        let audio = format!("https://content.{base_url}/{audio_key}");
        let filename = &audio_file["filename"];
        let extension = filename
            .as_str()
            .ok_or("Filename is missing from API response")?
            .split('.')
            .next_back()
            .ok_or(format!("Failed to parse extension from JSON: {filename}"))?
            .to_owned();
        Ok((audio, title, extension))
    }
}

impl Parser for AudiochanParser {
    fn audio(&self) -> &str {
        &self.audio
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn extension(&self) -> &str {
        &self.extension
    }
}

pub struct WhypParser {
    audio: String,
    title: String,
    extension: String,
}

impl WhypParser {
    pub async fn new(url: &str) -> Result<Self, String> {
        let (audio, title, extension) = Self::parse_info(url).await?;
        Ok(Self {
            audio,
            title,
            extension,
        })
    }

    async fn parse_info(url: &str) -> Result<(String, String, String), String> {
        let id = {
            let mut slash_parts = url.split('/');
            let (_slug, id) = (slash_parts.next_back(), slash_parts.next_back());
            id.ok_or(format!("Failed to parse id from URL: {}", url))?
        };
        let token = url
            .split('?')
            .next_back()
            .ok_or(format!("Failed to parse token from URL: {}", url))?;
        let base_url = "https://api.whyp.it";
        let body = reqwest::get(format!("{base_url}/api/tracks/{id}?{token}"))
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        let json: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let track = &json["track"];
        let title = track["title"]
            .as_str()
            .ok_or("Title is missing from API response")?
            .to_owned();
        let audio = {
            let audio_lossless = track["lossless_url"].as_str();
            let audio_lossy = track["lossy_url"].as_str();
            audio_lossless.or(audio_lossy)
        }
        .ok_or("Audio URL is missing from API response")?
        .to_owned();
        let extension = audio
            .split('?')
            .next()
            .ok_or(format!(
                "Failed to get file extension from audio URL: {audio}",
            ))?
            .split('.')
            .next_back()
            .ok_or(format!(
                "Failed to get file extension from audio URL: {audio}"
            ))?
            .to_owned();
        Ok((audio, title, extension))
    }
}

impl Parser for WhypParser {
    fn audio(&self) -> &str {
        &self.audio
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn extension(&self) -> &str {
        &self.extension
    }
}

async fn info_from_page(
    url: &str,
    audio_regex: &str,
    title_selector: &str,
) -> Result<(String, String, String), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch page {}: {}", url, e))?;
    let html = response
        .text()
        .await
        .map_err(|e| format!("Failed to parse html to text {}: {}", url, e))?;
    let audio: String = Regex::new(audio_regex)
        .map_err(|e| e.to_string())
        .and_then(|re| {
            re.captures(&html)
                .ok_or(format!("Failed to find valid audio url: {}", url))
        })
        .and_then(|caps| {
            caps.get(0)
                .ok_or(format!("Page contains no valid audio url: {}", url))
        })?
        .as_str()
        .to_owned();
    let extension = audio
        .split('.')
        .next_back()
        .ok_or(format!(
            "Audio URL contains no valid file extension: {audio}"
        ))?
        .to_owned();
    let document = Html::parse_document(&html);
    let raw_title: String = Selector::parse(title_selector)
        .map_err(|e| e.to_string())
        .and_then(|selector| {
            document
                .select(&selector)
                .next()
                .ok_or(format!("Page does not contain title: {}", url))
        })?
        .text()
        .collect();
    let title = Regex::new(r"(\[.+?\])")
        .map_err(|e| e.to_string())?
        .replace_all(&raw_title, "")
        .trim()
        .to_owned();
    Ok((audio, title, extension))
}
