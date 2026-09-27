use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub file_path: String,
    pub format_name: String,
    pub duration_seconds: f64,
    pub file_size_bytes: u64,
    pub overall_bitrate_kbps: u32,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
    pub subtitle_streams: Vec<SubtitleStreamInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    pub index: usize,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub pixel_format: String,
    pub bitrate_kbps: Option<u32>,
    pub frame_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamInfo {
    pub index: usize,
    pub codec: String,
    pub channels: u32,
    pub channel_layout: String,
    pub sample_rate_hz: u32,
    pub bitrate_kbps: Option<u32>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleStreamInfo {
    pub index: usize,
    pub codec: String,
    pub language: Option<String>,
    pub title: Option<String>,
}

/// Probe a media file using ffprobe JSON output
pub fn probe_file<P: AsRef<Path>>(path: P) -> Result<MediaInfo> {
    let p = path.as_ref();
    if !p.exists() {
        return Err(anyhow!("File not found: {}", p.display()));
    }

    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            p.to_str().unwrap_or_default(),
        ])
        .output()
        .context("Failed to execute ffprobe. Ensure ffmpeg/ffprobe is installed.")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffprobe failed on {}: {}", p.display(), err));
    }

    let json_val: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse ffprobe JSON output")?;

    parse_ffprobe_json(p.to_str().unwrap_or_default(), &json_val)
}

fn parse_ffprobe_json(file_path: &str, v: &serde_json::Value) -> Result<MediaInfo> {
    let format = v.get("format").ok_or_else(|| anyhow!("No format section in ffprobe"))?;
    let format_name = format.get("format_name").and_then(|s| s.as_str()).unwrap_or("unknown").to_string();
    
    let duration_seconds: f64 = format
        .get("duration")
        .and_then(|d| d.as_str())
        .and_then(|d| d.parse().ok())
        .unwrap_or(0.0);

    let file_size_bytes: u64 = format
        .get("size")
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let overall_bitrate_kbps: u32 = format
        .get("bit_rate")
        .and_then(|b| b.as_str())
        .and_then(|b| b.parse::<u64>().ok())
        .map(|bps| (bps / 1000) as u32)
        .unwrap_or(0);

    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();
    let mut subtitle_streams = Vec::new();

    if let Some(streams) = v.get("streams").and_then(|s| s.as_array()) {
        for s in streams {
            let codec_type = s.get("codec_type").and_then(|c| c.as_str()).unwrap_or("");
            let index = s.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
            let codec_name = s.get("codec_name").and_then(|c| c.as_str()).unwrap_or("unknown").to_string();

            match codec_type {
                "video" => {
                    let width = s.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                    let height = s.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
                    let pixel_format = s.get("pix_fmt").and_then(|p| p.as_str()).unwrap_or("yuv420p").to_string();
                    let fps = parse_fps(s.get("r_frame_rate").and_then(|r| r.as_str()).unwrap_or("30/1"));
                    let bitrate_kbps = s.get("bit_rate")
                        .and_then(|b| b.as_str())
                        .and_then(|b| b.parse::<u64>().ok())
                        .map(|bps| (bps / 1000) as u32);
                    let frame_count = s.get("nb_frames")
                        .and_then(|f| f.as_str())
                        .and_then(|f| f.parse::<u64>().ok());

                    video_streams.push(VideoStreamInfo {
                        index,
                        codec: codec_name,
                        width,
                        height,
                        fps,
                        pixel_format,
                        bitrate_kbps,
                        frame_count,
                    });
                }
                "audio" => {
                    let channels = s.get("channels").and_then(|c| c.as_u64()).unwrap_or(2) as u32;
                    let channel_layout = s.get("channel_layout").and_then(|l| l.as_str()).unwrap_or("stereo").to_string();
                    let sample_rate_hz = s.get("sample_rate")
                        .and_then(|r| r.as_str())
                        .and_then(|r| r.parse().ok())
                        .unwrap_or(48000);
                    let bitrate_kbps = s.get("bit_rate")
                        .and_then(|b| b.as_str())
                        .and_then(|b| b.parse::<u64>().ok())
                        .map(|bps| (bps / 1000) as u32);
                    let language = s.get("tags")
                        .and_then(|t| t.get("language"))
                        .and_then(|l| l.as_str())
                        .map(|s| s.to_string());

                    audio_streams.push(AudioStreamInfo {
                        index,
                        codec: codec_name,
                        channels,
                        channel_layout,
                        sample_rate_hz,
                        bitrate_kbps,
                        language,
                    });
                }
                "subtitle" => {
                    let language = s.get("tags")
                        .and_then(|t| t.get("language"))
                        .and_then(|l| l.as_str())
                        .map(|s| s.to_string());
                    let title = s.get("tags")
                        .and_then(|t| t.get("title"))
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string());

                    subtitle_streams.push(SubtitleStreamInfo {
                        index,
                        codec: codec_name,
                        language,
                        title,
                    });
                }
                _ => {}
            }
        }
    }

    Ok(MediaInfo {
        file_path: file_path.to_string(),
        format_name,
        duration_seconds,
        file_size_bytes,
        overall_bitrate_kbps,
        video_streams,
        audio_streams,
        subtitle_streams,
    })
}

fn parse_fps(rate: &str) -> f64 {
    if let Some((num, den)) = rate.split_once('/') {
        if let (Ok(n), Ok(d)) = (num.parse::<f64>(), den.parse::<f64>()) {
            if d != 0.0 {
                return n / d;
            }
        }
    }
    30.0
}
