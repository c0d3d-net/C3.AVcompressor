use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;
use colored::Colorize;
use crate::npu::AiStreamOptimizer;

#[derive(Debug, Clone)]
pub struct AudioExtractOptions {
    pub format: String,             // aac, ac3, dts, wav, mp3, flac, opus
    pub track_index: Option<usize>, // None = default first audio track
    pub channels: Option<u32>,      // 1 (mono), 2 (stereo), 6 (5.1)
    pub sample_rate: Option<u32>,   // 44100, 48000, 96000
    pub bitrate_kbps: Option<u32>,  // Bitrate
    pub stream_copy: bool,          // Lossless extraction
}

#[derive(Debug, Clone)]
pub struct VideoExtractOptions {
    pub stream_copy: bool,
    pub video_codec: Option<String>,
}

pub struct StreamExtractor {
    ai_optimizer: AiStreamOptimizer,
}

impl StreamExtractor {
    pub fn new() -> Self {
        Self {
            ai_optimizer: AiStreamOptimizer::new(),
        }
    }

    /// Extract an audio stream from an input video or audio container
    pub fn extract_audio<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input: P,
        output: Q,
        opts: &AudioExtractOptions,
    ) -> Result<()> {
        let in_path = input.as_ref();
        let out_path = output.as_ref();

        if !in_path.exists() {
            return Err(anyhow!("Input file not found: {}", in_path.display()));
        }

        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);
        cmd.args(["-i", in_path.to_str().unwrap()]);

        // Select audio map
        if let Some(track) = opts.track_index {
            cmd.args(["-map", &format!("0:a:{}", track)]);
        } else {
            cmd.args(["-map", "0:a:0?"]);
        }

        // Disable video and subtitles
        cmd.args(["-vn", "-sn"]);

        if opts.stream_copy {
            println!("{}", "⚡ Using high-speed lossless stream copy for audio extraction...".cyan());
            cmd.args(["-c:a", "copy"]);
        } else {
            let fmt_lower = opts.format.to_lowercase();
            let codec = match fmt_lower.as_str() {
                "mp3" | "mpeg3" => "libmp3lame",
                "aac" => "aac",
                "ac3" => "ac3",
                "eac3" => "eac3",
                "dts" | "dca" => "dca",
                "wav" => "pcm_s16le",
                "flac" => "flac",
                "opus" => "libopus",
                other => other,
            };

            cmd.args(["-c:a", codec]);

            // Determine bitrate with AI optimization if applicable
            let target_channels = opts.channels.unwrap_or(2);
            let target_bitrate = self.ai_optimizer.optimize_audio_bitrate(
                &opts.format,
                target_channels,
                opts.bitrate_kbps,
            );

            if !matches!(opts.format.to_lowercase().as_str(), "wav" | "flac") {
                cmd.args(["-b:a", &format!("{}k", target_bitrate)]);
            }

            if let Some(ch) = opts.channels {
                cmd.args(["-ac", &ch.to_string()]);
            }

            if let Some(sr) = opts.sample_rate {
                cmd.args(["-ar", &sr.to_string()]);
            }
        }

        cmd.arg(out_path.to_str().unwrap());

        let status = cmd.status().context("Failed to run ffmpeg audio extractor")?;
        if !status.success() {
            return Err(anyhow!("Audio extraction failed with exit code: {:?}", status.code()));
        }

        Ok(())
    }

    /// Extract pure video stream (stripping audio)
    pub fn extract_video<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input: P,
        output: Q,
        opts: &VideoExtractOptions,
    ) -> Result<()> {
        let in_path = input.as_ref();
        let out_path = output.as_ref();

        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);
        cmd.args(["-i", in_path.to_str().unwrap()]);
        cmd.args(["-an", "-sn"]);

        if opts.stream_copy {
            cmd.args(["-c:v", "copy"]);
        } else if let Some(ref codec) = opts.video_codec {
            cmd.args(["-c:v", codec]);
        }

        cmd.arg(out_path.to_str().unwrap());

        let status = cmd.status().context("Failed to run ffmpeg video extractor")?;
        if !status.success() {
            return Err(anyhow!("Video extraction failed with exit code: {:?}", status.code()));
        }

        Ok(())
    }
}

impl Default for StreamExtractor {
    fn default() -> Self {
        Self::new()
    }
}
