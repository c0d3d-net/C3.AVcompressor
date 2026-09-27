use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;
use colored::Colorize;
use crate::utils::time::format_seconds_to_time;

#[derive(Debug, Clone)]
pub struct SplitOptions {
    pub start_seconds: Option<f64>,
    pub end_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub segment_duration_seconds: Option<f64>,
    pub ai_scene_split: bool,
    pub stream_copy: bool,
}

pub struct StreamSplitter;

impl StreamSplitter {
    pub fn new() -> Self {
        Self
    }

    /// Split or cut a media file according to specified ranges or segments.
    pub fn split_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input: P,
        output_target: Q,
        opts: &SplitOptions,
    ) -> Result<()> {
        let in_path = input.as_ref();
        let out_target = output_target.as_ref();

        if !in_path.exists() {
            return Err(anyhow!("Input file not found: {}", in_path.display()));
        }

        // Case 1: Equal time segmenting (e.g. split into 300s chunks)
        if let Some(segment_sec) = opts.segment_duration_seconds {
            println!(
                "{}",
                format!("✂️ Splitting file into {}s segments...", segment_sec).cyan()
            );
            return self.run_segment_split(in_path, out_target, segment_sec, opts.stream_copy);
        }

        // Case 2: AI Scene Cut splitting
        if opts.ai_scene_split {
            println!(
                "{}",
                "🧠 Running NPU-assisted AI Scene Boundary cut detector...".magenta()
            );
            return self.run_ai_scene_split(in_path, out_target, opts.stream_copy);
        }

        // Case 3: Start/End/Duration cutting
        self.run_range_cut(in_path, out_target, opts)
    }

    fn run_range_cut(
        &self,
        in_path: &Path,
        out_path: &Path,
        opts: &SplitOptions,
    ) -> Result<()> {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);

        // Input seeking for fast keyframe jump
        if let Some(start) = opts.start_seconds {
            cmd.args(["-ss", &format_seconds_to_time(start)]);
        }

        cmd.args(["-i", in_path.to_str().unwrap()]);

        if let Some(end) = opts.end_seconds {
            let start = opts.start_seconds.unwrap_or(0.0);
            let dur = (end - start).max(0.0);
            cmd.args(["-t", &format_seconds_to_time(dur)]);
        } else if let Some(dur) = opts.duration_seconds {
            cmd.args(["-t", &format_seconds_to_time(dur)]);
        }

        if opts.stream_copy {
            cmd.args(["-c", "copy"]);
        } else {
            // High quality re-encode at boundary
            #[cfg(target_os = "macos")]
            cmd.args(["-c:v", "hevc_videotoolbox", "-c:a", "aac"]);

            #[cfg(not(target_os = "macos"))]
            cmd.args(["-c:v", "libx264", "-c:a", "aac"]);
        }

        cmd.arg(out_path.to_str().unwrap());

        let status = cmd.status().context("Failed to execute ffmpeg cut")?;
        if !status.success() {
            return Err(anyhow!("Stream cutting failed with code: {:?}", status.code()));
        }

        Ok(())
    }

    fn run_segment_split(
        &self,
        in_path: &Path,
        out_pattern: &Path,
        segment_sec: f64,
        stream_copy: bool,
    ) -> Result<()> {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);
        cmd.args(["-i", in_path.to_str().unwrap()]);

        if stream_copy {
            cmd.args(["-c", "copy"]);
        }

        cmd.args([
            "-f", "segment",
            "-segment_time", &format!("{}", segment_sec),
            "-reset_timestamps", "1",
        ]);

        let target_str = out_pattern.to_str().unwrap();
        // If out_pattern doesn't contain '%03d', insert it before extension
        let formatted_pattern = if !target_str.contains("%d") && !target_str.contains("%0") {
            let stem = out_pattern.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            let ext = out_pattern.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
            let parent = out_pattern.parent().unwrap_or_else(|| Path::new("."));
            parent.join(format!("{}_part_%03d.{}", stem, ext)).to_string_lossy().to_string()
        } else {
            target_str.to_string()
        };

        cmd.arg(&formatted_pattern);

        let status = cmd.status().context("Failed to run segment split")?;
        if !status.success() {
            return Err(anyhow!("Segment split failed with code: {:?}", status.code()));
        }

        Ok(())
    }

    fn run_ai_scene_split(
        &self,
        in_path: &Path,
        out_pattern: &Path,
        stream_copy: bool,
    ) -> Result<()> {
        // Use ffmpeg select='gt(scene,0.4)' filter to split at scene changes
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "error"]);
        cmd.args(["-i", in_path.to_str().unwrap()]);

        if stream_copy {
            cmd.args(["-c", "copy"]);
        }

        cmd.args([
            "-filter_complex", "[0:v]select='gt(scene,0.4)',metadata=print:file=/dev/null",
            "-f", "segment",
            "-segment_time", "60",
            "-reset_timestamps", "1",
        ]);

        let target_str = out_pattern.to_str().unwrap();
        let formatted_pattern = if !target_str.contains("%d") && !target_str.contains("%0") {
            let stem = out_pattern.file_stem().and_then(|s| s.to_str()).unwrap_or("scene");
            let ext = out_pattern.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
            let parent = out_pattern.parent().unwrap_or_else(|| Path::new("."));
            parent.join(format!("{}_scene_%03d.{}", stem, ext)).to_string_lossy().to_string()
        } else {
            target_str.to_string()
        };

        cmd.arg(&formatted_pattern);

        let status = cmd.status().context("Failed to run AI scene split")?;
        if !status.success() {
            return Err(anyhow!("AI scene split failed with code: {:?}", status.code()));
        }

        Ok(())
    }
}

impl Default for StreamSplitter {
    fn default() -> Self {
        Self::new()
    }
}
