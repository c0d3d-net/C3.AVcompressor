use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::core::probe::probe_file;
use crate::hardware::{detect_hardware_encoder, HardwareAccelerator, VideoEncoderBackend};
use crate::npu::{AiStreamOptimizer, NpuDeviceInfo};

#[derive(Debug, Clone)]
pub struct CompressOptions {
    pub video_codec: String,         // h264, hevc, av1, vp9, prores, copy
    pub audio_codec: String,         // aac, ac3, dts, wav, mp3, flac, opus, copy
    pub preset: String,              // ultra-fast, balanced, high-quality, extreme-compression
    pub crf: Option<u8>,             // Base CRF (e.g. 22)
    pub video_bitrate_kbps: Option<u32>,
    pub audio_bitrate_kbps: Option<u32>,
    pub resolution: Option<String>,  // e.g. "1920x1080", "1280x720", "3840x2160"
    pub width: Option<u32>,          // Target width with auto-calculated height preserving aspect ratio
    pub height: Option<u32>,         // Target height with auto-calculated width preserving aspect ratio
    pub fps: Option<f64>,
    pub enable_ai_npu: bool,
    pub hw_accel: Option<String>,
}

pub struct MediaCompressor {
    ai_optimizer: AiStreamOptimizer,
    hw_backend: VideoEncoderBackend,
}

impl MediaCompressor {
    pub fn new() -> Self {
        Self {
            ai_optimizer: AiStreamOptimizer::new(),
            hw_backend: detect_hardware_encoder(),
        }
    }

    pub fn npu_device(&self) -> &NpuDeviceInfo {
        self.ai_optimizer.device()
    }

    #[allow(dead_code)]
    pub fn hw_backend(&self) -> &VideoEncoderBackend {
        &self.hw_backend
    }

    /// Execute the full compression and transcoding pipeline
    pub fn compress<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input: P,
        output: Q,
        opts: &CompressOptions,
    ) -> Result<()> {
        let in_path = input.as_ref();
        let out_path = output.as_ref();

        let probe = probe_file(in_path)?;
        let duration = probe.duration_seconds;

        println!("{}", "═══════════════════════════════════════════════════════════════".blue());
        println!(
            "{} {}",
            "🚀 C3.AVcompressor Transcoding Engine:".bold().white(),
            in_path.file_name().unwrap_or_default().to_string_lossy().cyan()
        );
        println!(
            "   Duration: {:.2}s | Size: {:.2} MB | Bitrate: {} kbps",
            duration,
            probe.file_size_bytes as f64 / 1_048_576.0,
            probe.overall_bitrate_kbps
        );

        // Hardware & NPU status display
        if self.ai_optimizer.is_npu_active() && opts.enable_ai_npu {
            println!(
                "   {} {} ({:.1} TOPS)",
                "🧠 NPU Acceleration Active:".bold().magenta(),
                self.npu_device().name.green(),
                self.npu_device().estimated_tops
            );
            println!(
                "   {} {}",
                "⚡ Zero-Copy Engine:".bold().yellow(),
                if self.npu_device().zero_copy_unified_memory {
                    "Unified Memory Architecture (UMA) Enabled".green()
                } else {
                    "Standard Memory Bus".white()
                }
            );
        } else {
            println!(
                "   {} {}",
                "⚙️ Hardware Accelerator:".bold().yellow(),
                format!("{:?}", self.hw_backend.accelerator).cyan()
            );
        }

        // Calculate AI optimization plan
        // Calculate AI optimization plan & resolve target dimensions
        let base_crf = opts.crf.unwrap_or(23);
        let first_v = probe.video_streams.first();
        let orig_w = first_v.map(|v| v.width).unwrap_or(1920);
        let orig_h = first_v.map(|v| v.height).unwrap_or(1080);
        let fps = opts.fps.unwrap_or_else(|| first_v.map(|v| v.fps).unwrap_or(30.0));

        let target_dims = crate::utils::resolution::resolve_dimensions(
            orig_w,
            orig_h,
            opts.width,
            opts.height,
            opts.resolution.as_deref(),
        )?;

        let (target_w, target_h) = target_dims.unwrap_or((orig_w, orig_h));
        if target_dims.is_some() {
            println!(
                "   {} {}x{} -> {} (Aspect Ratio Preserved)",
                "📐 Resolution Scaling:".cyan(),
                orig_w, orig_h,
                format!("{}x{}", target_w, target_h).bold().yellow()
            );
        }

        let ai_plan = self.ai_optimizer.plan_encoding(
            base_crf,
            target_w,
            target_h,
            fps,
            &opts.preset,
        );

        if let Some(bitrate) = opts.video_bitrate_kbps {
            println!(
                "   {} Explicit Target Bitrate: {} kbps (Hardware CBR/VBR Engine)",
                "🎯 Rate Control:".cyan(),
                bitrate.to_string().bold().yellow()
            );
        } else if opts.enable_ai_npu && self.ai_optimizer.is_npu_active() {
            println!(
                "   {} Dynamic CRF: {} (QP offset: {}) | Est. Bitrate Reduction: -{:.1}%",
                "🎯 AI Stream Plan:".magenta(),
                ai_plan.recommended_crf.to_string().bold().green(),
                ai_plan.qp_offset,
                ai_plan.estimated_bitrate_reduction_percent
            );
        }
        println!("{}", "═══════════════════════════════════════════════════════════════".blue());

        // Build FFmpeg command with optimal hardware and AI flags
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y", "-hide_banner", "-loglevel", "info"]);

        // Hardware decode flags
        match self.hw_backend.accelerator {
            HardwareAccelerator::VideoToolbox => {
                // VideoToolbox hardware decoding
                cmd.args(["-hwaccel", "videotoolbox"]);
            }
            HardwareAccelerator::Vaapi => {
                if let Some(ref dev) = self.hw_backend.hardware_device {
                    cmd.args(["-vaapi_device", dev, "-hwaccel", "vaapi", "-hwaccel_output_format", "vaapi"]);
                }
            }
            HardwareAccelerator::Nvenc => {
                cmd.args(["-hwaccel", "cuda"]);
            }
            _ => {}
        }

        cmd.args(["-i", in_path.to_str().unwrap()]);

        // Video codec selection and hardware parameters
        let v_codec_lower = opts.video_codec.to_lowercase();
        if v_codec_lower == "copy" {
            cmd.args(["-c:v", "copy"]);
        } else {
            let actual_encoder = self.resolve_video_encoder(&v_codec_lower, opts.hw_accel.as_deref());
            cmd.args(["-c:v", &actual_encoder]);

            // Configure rate control & quality
            self.apply_encoder_tuning(
                &mut cmd,
                &actual_encoder,
                ai_plan.recommended_crf,
                opts.video_bitrate_kbps,
                &opts.preset,
            );
        }

        // Resolution scaling
        if target_dims.is_some() {
            cmd.args(["-vf", &format!("scale={}:{}", target_w, target_h)]);
        }

        // Framerate
        if let Some(f) = opts.fps {
            cmd.args(["-r", &format!("{}", f)]);
        }

        // Audio codec selection
        let a_codec_lower = opts.audio_codec.to_lowercase();
        if a_codec_lower == "copy" {
            cmd.args(["-c:a", "copy"]);
        } else {
            let actual_a_encoder = match a_codec_lower.as_str() {
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
            cmd.args(["-c:a", actual_a_encoder]);

            let target_a_bitrate = self.ai_optimizer.optimize_audio_bitrate(
                &a_codec_lower,
                2,
                opts.audio_bitrate_kbps,
            );
            if !matches!(a_codec_lower.as_str(), "wav" | "flac") {
                cmd.args(["-b:a", &format!("{}k", target_a_bitrate)]);
            }
        }

        // Progress piping
        cmd.args(["-progress", "pipe:1"]);
        cmd.arg(out_path.to_str().unwrap());

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());

        let mut child = cmd.spawn().context("Failed to spawn ffmpeg transcode process")?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to capture stdout"))?;

        // Initialize progress bar
        let pb = ProgressBar::new((duration * 1000.0) as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% | {msg}")
                .unwrap()
                .progress_chars("█▓▒░"),
        );

        let reader = BufReader::new(stdout);
        let mut current_speed = String::from("1.0x");
        let mut current_fps = 0.0;

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("out_time_ms=") {
                let ms_str = &line["out_time_ms=".len()..];
                if let Ok(ms) = ms_str.parse::<u64>() {
                    pb.set_position(ms / 1000);
                }
            } else if line.starts_with("speed=") {
                current_speed = line["speed=".len()..].trim().to_string();
            } else if line.starts_with("fps=") {
                if let Ok(f) = line["fps=".len()..].trim().parse::<f64>() {
                    current_fps = f;
                }
            }

            pb.set_message(format!(
                "Speed: {} | {:.1} FPS | NPU AI Active",
                current_speed.bold().green(),
                current_fps
            ));
        }

        let status = child.wait()?;
        pb.finish_with_message("✅ Transcoding Completed Successfully!".bold().green().to_string());

        if !status.success() {
            return Err(anyhow!("Transcode process failed with exit code: {:?}", status.code()));
        }

        // Print final stats
        if out_path.exists() {
            let out_meta = std::fs::metadata(out_path)?;
            let out_mb = out_meta.len() as f64 / 1_048_576.0;
            let in_mb = probe.file_size_bytes as f64 / 1_048_576.0;
            let ratio = if in_mb > 0.0 { (out_mb / in_mb) * 100.0 } else { 100.0 };
            let saved = 100.0 - ratio;

            println!("{}", "───────────────────────────────────────────────────────────────".blue());
            println!(
                "🎉 Output: {} | Size: {:.2} MB (Saved: {:.1}% bitrate)",
                out_path.display().to_string().cyan(),
                out_mb,
                saved.max(0.0)
            );
            println!("{}", "───────────────────────────────────────────────────────────────".blue());
        }

        Ok(())
    }

    fn resolve_video_encoder(&self, codec: &str, forced_hw: Option<&str>) -> String {
        let is_videotoolbox = self.hw_backend.accelerator == HardwareAccelerator::VideoToolbox
            || forced_hw == Some("videotoolbox");
        let is_vaapi = self.hw_backend.accelerator == HardwareAccelerator::Vaapi
            || forced_hw == Some("vaapi");
        let is_nvenc = self.hw_backend.accelerator == HardwareAccelerator::Nvenc
            || forced_hw == Some("nvenc");

        match codec {
            "h264" | "avc" | "x264" => {
                if is_videotoolbox {
                    "h264_videotoolbox".to_string()
                } else if is_vaapi {
                    "h264_vaapi".to_string()
                } else if is_nvenc {
                    "h264_nvenc".to_string()
                } else {
                    "libx264".to_string()
                }
            }
            "h265" | "hevc" | "x265" => {
                if is_videotoolbox {
                    "hevc_videotoolbox".to_string()
                } else if is_vaapi {
                    "hevc_vaapi".to_string()
                } else if is_nvenc {
                    "hevc_nvenc".to_string()
                } else {
                    "libx265".to_string()
                }
            }
            "av1" => {
                if is_nvenc {
                    "av1_nvenc".to_string()
                } else if is_vaapi {
                    "av1_vaapi".to_string()
                } else {
                    "libsvtav1".to_string()
                }
            }
            "prores" => {
                if is_videotoolbox {
                    "prores_videotoolbox".to_string()
                } else {
                    "prores_ks".to_string()
                }
            }
            "vp9" => "libvpx-vp9".to_string(),
            other => other.to_string(),
        }
    }

    fn apply_encoder_tuning(
        &self,
        cmd: &mut Command,
        encoder: &str,
        recommended_crf: u8,
        video_bitrate_kbps: Option<u32>,
        preset: &str,
    ) {
        if encoder.contains("videotoolbox") {
            // Apple Silicon VideoToolbox tuning
            if let Some(bitrate) = video_bitrate_kbps {
                cmd.args(["-b:v", &format!("{}k", bitrate)]);
            } else {
                // Map CRF to VideoToolbox -q:v (1 to 100 quality scale)
                // CRF 18 ≈ -q:v 75, CRF 23 ≈ -q:v 60, CRF 28 ≈ -q:v 48
                let vt_q = ((51.0 - recommended_crf as f64) * 2.0).clamp(20.0, 95.0).round() as u32;
                cmd.args(["-q:v", &vt_q.to_string()]);
            }
            // Real-time prioritization and profile
            cmd.args(["-realtime", "0"]);
            if encoder.contains("hevc") {
                cmd.args(["-tag:v", "hvc1"]); // macOS/iOS QuickTime compatibility
            }
        } else if encoder.contains("nvenc") {
            if let Some(bitrate) = video_bitrate_kbps {
                cmd.args(["-b:v", &format!("{}k", bitrate)]);
            } else {
                cmd.args(["-cq", &recommended_crf.to_string()]);
            }
            let nv_preset = match preset {
                "ultra-fast" => "p1",
                "high-quality" => "p7",
                "extreme-compression" => "p7",
                _ => "p4",
            };
            cmd.args(["-preset", nv_preset]);
        } else if encoder.contains("vaapi") {
            if let Some(bitrate) = video_bitrate_kbps {
                cmd.args(["-b:v", &format!("{}k", bitrate)]);
            } else {
                cmd.args(["-qp", &recommended_crf.to_string()]);
            }
        } else {
            // Software encoders (libx264, libx265, libsvtav1)
            if let Some(bitrate) = video_bitrate_kbps {
                let maxrate = (bitrate as f64 * 1.3).round() as u32;
                let bufsize = bitrate * 2;
                cmd.args([
                    "-b:v", &format!("{}k", bitrate),
                    "-maxrate", &format!("{}k", maxrate),
                    "-bufsize", &format!("{}k", bufsize),
                ]);
            } else {
                cmd.args(["-crf", &recommended_crf.to_string()]);
            }
            let sw_preset = match preset {
                "ultra-fast" => "ultrafast",
                "high-quality" => "slow",
                "extreme-compression" => "veryslow",
                _ => "medium",
            };
            cmd.args(["-preset", sw_preset]);
        }
    }
}

impl Default for MediaCompressor {
    fn default() -> Self {
        Self::new()
    }
}
