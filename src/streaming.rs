#![allow(dead_code)]

use std::process::Command;

#[derive(Debug, Clone)]
pub struct StreamFormat {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: Option<i32>,
    pub has_audio: bool,
    pub has_video: bool,
}

pub struct StreamInfo {
    pub formats: Vec<StreamFormat>,
}

/// Extract video info from a URL using yt-dlp as JSON
pub fn extract_info(url: &str) -> Option<StreamInfo> {
    let output = Command::new("yt-dlp")
        .args([
            "--dump-json",
            "--no-playlist",
            "--no-download",
            url,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        log::error!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr));
        return None;
    }

    let json_str = String::from_utf8(output.stdout).ok()?;
    let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    let formats = json["formats"]
        .as_array()?
        .iter()
        .filter_map(|f| {
            let url = f["url"].as_str()?.to_string();
            let width = f["width"].as_u64().map(|v| v as u32);
            let height = f["height"].as_u64().map(|v| v as u32);
            let quality = f["quality"].as_i64().map(|v| v as i32);
            let has_audio = f["acodec"].as_str() != Some("none");
            let has_video = f["vcodec"].as_str() != Some("none");
            Some(StreamFormat {
                url,
                width,
                height,
                quality,
                has_audio,
                has_video,
            })
        })
        .collect();

    Some(StreamInfo { formats })
}

/// Get the best video-only format closest to target height
pub fn get_optimal_video(formats: &[StreamFormat], target_height: u32) -> Option<&StreamFormat> {
    formats
        .iter()
        .filter(|f| f.has_video && !f.has_audio)
        .chain(formats.iter().filter(|f| f.has_video && f.has_audio))
        .min_by_key(|f| f.height.map_or(u32::MAX, |h| h.abs_diff(target_height)))
}

/// Get the best audio-only format
pub fn get_best_audio(formats: &[StreamFormat]) -> Option<&StreamFormat> {
    formats
        .iter()
        .filter(|f| !f.has_video && f.has_audio)
        .chain(formats.iter().filter(|f| f.has_video && f.has_audio))
        .max_by_key(|f| f.quality.unwrap_or(-1))
}

/// Check if a URL is a streamable URL (YouTube, etc.)
pub fn is_stream_url(url: &str) -> bool {
    let supported_hosts = [
        "youtube.com", "youtu.be", "twitch.tv", "vimeo.com",
        "dailymotion.com", "facebook.com", "instagram.com",
    ];
    supported_hosts.iter().any(|&host| url.contains(host))
}
