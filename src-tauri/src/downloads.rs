use crate::parsers::{AudiochanParser, Parser, SoundgasmParser, VocarooParser, WhypParser};
use regex::Regex;
use std::collections::HashMap;

#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum DownloadStatus {
    #[default]
    Initial,
    Downloading,
    Completed,
    Failed,
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
struct DownloadInfo {
    audio: String,
    title: String,
    extension: String,
    headers: HashMap<String, String>,
}

impl DownloadInfo {
    fn new(
        audio: String,
        title: String,
        extension: String,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            audio,
            title,
            extension,
            headers,
        }
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DownloadInput {
    url: String,
    op: String,
    sub: String,
}

impl DownloadInput {
    pub fn url(&self) -> &str {
        &self.url
    }
    // Hostname regex pattern from URI spec: https://www.rfc-editor.org/rfc/rfc3986#appendix-B
    async fn parse_info(&self) -> Result<DownloadInfo, String> {
        let url_captures =
            Regex::new(r"^(([^:\/?#]+):)?(\/\/([^\/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?")
                .map_err(|e| e.to_string())
                .and_then(|re| {
                    re.captures(&self.url)
                        .ok_or(format!("Failed to match hostname: {}", &self.url))
                })?;
        let hostname = url_captures
            .get(4)
            .ok_or(format!("URL contains no valid hostname: {}", &self.url))?
            .as_str()
            .to_owned();
        let parser: Box<dyn Parser> = match () {
            () if hostname.contains("soundgasm.net") => {
                Box::new(SoundgasmParser::new(self.url()).await?)
            }
            () if hostname.contains("vocaroo.com") => {
                let id = url_captures
                    .get(5)
                    .ok_or(format!("URL contains no valid id: {}", &self.url))?
                    .as_str();
                Box::new(VocarooParser::new(id))
            }
            () if hostname.contains("audiochan.com") => {
                Box::new(AudiochanParser::new(self.url()).await?)
            }
            () if hostname.contains("whyp.it") => Box::new(WhypParser::new(self.url()).await?),
            () => {
                return Err(format!(
                    "URL contains invalid or unsupported host: {}",
                    &self.url
                ));
            }
        };
        Ok(DownloadInfo::new(
            parser.audio().to_owned(),
            parser.title().to_owned(),
            parser.extension().to_owned(),
            parser.headers(),
        ))
    }

    pub async fn parse_input(self, id: usize) -> Result<DownloadItem, String> {
        let info = self.parse_info().await?;
        Ok(DownloadItem::new(self, info, id))
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DownloadItem {
    input: DownloadInput,
    info: DownloadInfo,
    status: DownloadStatus,
    id: usize,
    failure: Option<String>,
}

impl DownloadItem {
    fn new(input: DownloadInput, info: DownloadInfo, id: usize) -> Self {
        Self {
            input,
            info,
            status: DownloadStatus::Initial,
            id,
            failure: None,
        }
    }

    pub async fn parse_input(self) -> Result<Self, String> {
        self.input.parse_input(self.id).await
    }

    pub fn url(&self) -> &str {
        &self.input.url
    }

    pub fn op(&self) -> &str {
        &self.input.op
    }

    pub fn sub(&self) -> &str {
        &self.input.sub
    }

    pub fn audio(&self) -> &str {
        &self.info.audio
    }

    pub fn title(&self) -> &str {
        &self.info.title
    }

    pub fn set_status(&mut self, status: DownloadStatus) {
        self.status = status;
    }

    pub fn set_failure(&mut self, failure: Option<String>) {
        self.failure = failure;
    }

    pub fn is_completed(&self) -> bool {
        self.status == DownloadStatus::Completed
    }

    pub fn is_initial(&self) -> bool {
        self.status == DownloadStatus::Initial
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.info.headers
    }

    pub fn filename(&self) -> Result<String, String> {
        let filename = format!(
            "[{}] [{}] {}.{}",
            &self.input.sub, &self.input.op, &self.info.title, &self.info.extension,
        );
        Regex::new(r#"[<>:"/\\\?\*|]+"#)
            .map_err(|e| format!("Invalid regex pattern: {e}"))
            .map(|re| re.replace_all(&filename, "").trim().to_owned())
    }
}
