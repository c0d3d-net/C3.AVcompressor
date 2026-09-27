use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "c3avcompressor",
    author = "C3net Development",
    version = "0.1.0",
    about = "C3.AVcompressor - High-Performance AI/NPU Accelerated Media Compressor, Splitter & Extractor",
    long_about = "C3.AVcompressor is a next-generation media processing utility by C3net Development.\nIt harnesses Apple Silicon Neural Engine (ANE) and Intel/AMD NPUs with AI stream algorithms\nto achieve faster-than-CUDA transcoding, intelligent rate control, and instant lossless splitting/extraction."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose logging output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compress and transcode video/audio using NPU AI rate control and hardware encoders
    Compress(CompressArgs),

    /// Split or cut media streams by time ranges, segments, or AI scene cut detection
    Split(SplitArgs),

    /// Extract audio tracks (AAC, AC3, DTS, WAV, MP3, FLAC, Opus) or video streams
    Extract(ExtractArgs),

    /// Inspect media streams, codecs, bitrates, and metadata
    Probe(ProbeArgs),

    /// Display detected NPU hardware, TOPS capability, and accelerator status
    NpuStatus(NpuStatusArgs),

    /// Benchmark NPU throughput against standard encoders
    Benchmark(BenchmarkArgs),

    /// List and inspect available transcoding and track selection presets
    Presets {
        /// Filter presets by name or id
        #[arg(short, long)]
        query: Option<String>,
    },

    /// Generate shell auto-completion scripts (bash, zsh, fish, powershell)
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Args, Debug)]
pub struct CompressArgs {
    /// Input media file
    #[arg(short, long)]
    pub input: String,

    /// Output media file
    #[arg(short, long)]
    pub output: String,

    /// Load a named transcoding preset (e.g. "Web 1080p HEVC 4500k")
    #[arg(long = "transcode-preset", alias = "profile")]
    pub transcode_preset: Option<String>,

    /// Video codec (hevc, h264, av1, vp9, prores, copy)
    #[arg(long, default_value = "hevc")]
    pub video_codec: String,

    /// Audio codec (aac, ac3, dts, wav, mp3, flac, opus, copy)
    #[arg(long, default_value = "aac")]
    pub audio_codec: String,

    /// Compression preset
    #[arg(long, default_value = "balanced", value_enum)]
    pub preset: PresetChoice,

    /// Target CRF / Quality value (16-38, default: 23)
    #[arg(long)]
    pub crf: Option<u8>,

    /// Target video bitrate (e.g. 4500, 4500k, 5M)
    #[arg(long, short = 'b', alias = "b:v")]
    pub video_bitrate: Option<String>,

    /// Target audio bitrate (e.g. 192, 192k, 320k)
    #[arg(long, alias = "b:a")]
    pub audio_bitrate: Option<String>,

    /// Output resolution scaling (e.g. 1920x1080, 1280x-1, 1080p)
    #[arg(long)]
    pub resolution: Option<String>,

    /// Target video width with auto-calculated height preserving aspect ratio (e.g. 1920, 1280)
    #[arg(long, short = 'w')]
    pub width: Option<u32>,

    /// Target video height with auto-calculated width preserving aspect ratio (e.g. 1080, 720)
    #[arg(long)]
    pub height: Option<u32>,

    /// Target framerate (e.g. 24, 30, 60)
    #[arg(long)]
    pub fps: Option<f64>,

    /// Enable AI/NPU acceleration (defaults to true if NPU is present)
    #[arg(long, default_value_t = true)]
    pub ai_npu: bool,

    /// Force specific hardware accelerator (videotoolbox, vaapi, nvenc, cpu)
    #[arg(long)]
    pub hw_accel: Option<String>,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetChoice {
    UltraFast,
    Balanced,
    HighQuality,
    ExtremeCompression,
}

impl std::fmt::Display for PresetChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetChoice::UltraFast => write!(f, "ultra-fast"),
            PresetChoice::Balanced => write!(f, "balanced"),
            PresetChoice::HighQuality => write!(f, "high-quality"),
            PresetChoice::ExtremeCompression => write!(f, "extreme-compression"),
        }
    }
}

#[derive(Args, Debug)]
pub struct SplitArgs {
    /// Input media file
    #[arg(short, long)]
    pub input: String,

    /// Output file or output pattern (e.g. output_%03d.mp4)
    #[arg(short, long)]
    pub output: String,

    /// Start timestamp (e.g. 00:01:30 or 90s)
    #[arg(long)]
    pub start: Option<String>,

    /// End timestamp (e.g. 00:05:00)
    #[arg(long)]
    pub end: Option<String>,

    /// Duration to keep from start (e.g. 60s, 5m)
    #[arg(short, long)]
    pub duration: Option<String>,

    /// Split file into equal duration segments (e.g. 300s)
    #[arg(long)]
    pub segment_time: Option<String>,

    /// Split automatically at AI detected scene changes
    #[arg(long, default_value_t = false)]
    pub ai_scene_split: bool,

    /// Lossless stream copy mode (instant, no re-encoding)
    #[arg(long, default_value_t = true)]
    pub copy: bool,
}

#[derive(Args, Debug)]
pub struct ExtractArgs {
    /// Input media file
    #[arg(short, long)]
    pub input: String,

    /// Output extracted file (e.g. audio.mp3, video_only.mp4)
    #[arg(short, long)]
    pub output: String,

    /// Type of stream to extract (audio, video)
    #[arg(long, default_value = "audio")]
    pub stream_type: String,

    /// Target audio format (aac, ac3, dts, wav, mp3, flac, opus)
    #[arg(long, default_value = "mp3")]
    pub format: String,

    /// Specific track index (default: first track)
    #[arg(long)]
    pub track: Option<usize>,

    /// Audio channels (1 = mono, 2 = stereo, 6 = 5.1 surround)
    #[arg(long)]
    pub channels: Option<u32>,

    /// Sample rate in Hz (e.g. 44100, 48000, 96000)
    #[arg(long)]
    pub sample_rate: Option<u32>,

    /// Target audio bitrate (e.g. 320, 320k, 192k)
    #[arg(long, short = 'b', alias = "b:a")]
    pub bitrate: Option<String>,

    /// Lossless stream copy mode (fastest, keeps original codec)
    #[arg(long, default_value_t = false)]
    pub copy: bool,
}

#[derive(Args, Debug)]
pub struct ProbeArgs {
    /// Input media file to analyze
    #[arg(short, long)]
    pub input: String,

    /// Output full stream analysis as JSON
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct NpuStatusArgs {
    /// Output details in JSON format
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct BenchmarkArgs {
    /// Resolution to benchmark (1080p, 4k)
    #[arg(long, default_value = "1080p")]
    pub resolution: String,

    /// Duration in seconds for synthetic benchmark
    #[arg(long, default_value_t = 5)]
    pub duration: u32,
}
