use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResolutionOption {
    #[default]
    Original,
    Uhd4K,       // 3840x2160
    Fhd1080p,    // 1920x1080
    Hd720p,      // 1280x720
    Sd480p,      // 854x480
    CustomWidth, // -w target_width (preserves aspect ratio)
    CustomHeight,// --height target_height (preserves aspect ratio)
}

impl ResolutionOption {
    pub fn label(&self) -> &'static str {
        match self {
            ResolutionOption::Original => "Originalgröße beibehalten",
            ResolutionOption::Uhd4K => "4K UHD (3840x2160)",
            ResolutionOption::Fhd1080p => "Full HD 1080p (1920x1080)",
            ResolutionOption::Hd720p => "HD 720p (1280x720)",
            ResolutionOption::Sd480p => "SD 480p (854x480)",
            ResolutionOption::CustomWidth => "Breite anpassen (Höhe autom. proportional)",
            ResolutionOption::CustomHeight => "Höhe anpassen (Breite autom. proportional)",
        }
    }
}

/// Encoding configuration applied to a media item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    pub video_codec: String,         // hevc, h264, av1, vp9, prores, copy
    pub audio_codec: String,         // aac, ac3, dts, wav, mp3, flac, opus, copy
    pub preset: String,              // ultra-fast, balanced, high-quality, extreme-compression
    pub crf: u8,                     // 16..38, default 23
    pub use_bitrate: bool,
    pub video_bitrate_kbps: Option<u32>,
    pub audio_bitrate_kbps: u32,     // 128, 192, 256, 320
    pub resolution_mode: ResolutionOption,
    pub custom_width: Option<u32>,
    pub custom_height: Option<u32>,
    pub fps: Option<f64>,
    pub enable_ai_npu: bool,
    pub hw_accel: String,            // auto, videotoolbox, vaapi, nvenc, cpu
    pub container_override: Option<String>,
}

impl Default for EncodingConfig {
    fn default() -> Self {
        Self {
            video_codec: "hevc".to_string(),
            audio_codec: "aac".to_string(),
            preset: "balanced".to_string(),
            crf: 23,
            use_bitrate: false,
            video_bitrate_kbps: None,
            audio_bitrate_kbps: 256,
            resolution_mode: ResolutionOption::Original,
            custom_width: None,
            custom_height: None,
            fps: None,
            enable_ai_npu: true,
            hw_accel: "auto".to_string(),
            container_override: None,
        }
    }
}

/// Encoding Parameter Preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: EncodingConfig,
    pub is_builtin: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoPresetMode {
    All,
    FirstOnly,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioPresetMode {
    All,
    MatchingLanguagesOnly,
    FirstOnly,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtitlePresetMode {
    All,
    MatchingLanguagesOnly,
    ForcedOnly,
    None,
}

/// Track Selection Preset for automated stream filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub video_mode: VideoPresetMode,
    pub audio_mode: AudioPresetMode,
    pub audio_languages: Vec<String>,
    pub audio_fallback_to_first: bool,
    pub subtitle_mode: SubtitlePresetMode,
    pub subtitle_languages: Vec<String>,
    pub is_builtin: bool,
}

/// Preset Manager persisting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetStore {
    pub track_presets: Vec<TrackPreset>,
    pub encoding_presets: Vec<EncodingPreset>,
    pub active_track_preset_id: String,
    pub active_encoding_preset_id: String,
}

impl Default for PresetStore {
    fn default() -> Self {
        let track_presets = Self::default_track_presets();
        let encoding_presets = Self::default_encoding_presets();
        let active_track_preset_id = track_presets.get(1).map(|p| p.id.clone()).unwrap_or_default();
        let active_encoding_preset_id = encoding_presets[0].id.clone();

        Self {
            track_presets,
            encoding_presets,
            active_track_preset_id,
            active_encoding_preset_id,
        }
    }
}

impl PresetStore {
    pub fn load_or_default() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(store) = serde_json::from_str::<PresetStore>(&content) {
                        return store;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            fs::write(path, json)?;
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        let p_gui = PathBuf::from(home.clone()).join(".config").join("c3avcompressorgui").join("presets.json");
        if p_gui.exists() {
            return Some(p_gui);
        }
        Some(PathBuf::from(home).join(".config").join("c3avcompressor").join("presets.json"))
    }

    pub fn find_encoding_preset(&self, query: &str) -> Option<&EncodingPreset> {
        let q = query.trim().to_lowercase();
        // Exact ID match
        if let Some(p) = self.encoding_presets.iter().find(|p| p.id.to_lowercase() == q) {
            return Some(p);
        }
        // Exact name match
        if let Some(p) = self.encoding_presets.iter().find(|p| p.name.to_lowercase() == q) {
            return Some(p);
        }
        // Substring name match
        self.encoding_presets.iter().find(|p| p.name.to_lowercase().contains(&q))
    }

    pub fn default_track_presets() -> Vec<TrackPreset> {
        vec![
            TrackPreset {
                id: "track_passthrough".to_string(),
                name: "Alle Spuren beibehalten (100% Passthrough)".to_string(),
                description: "Behält alle Video-, Audio- und Untertitelspuren des Quellmaterials bei.".to_string(),
                video_mode: VideoPresetMode::All,
                audio_mode: AudioPresetMode::All,
                audio_languages: Vec::new(),
                audio_fallback_to_first: true,
                subtitle_mode: SubtitlePresetMode::All,
                subtitle_languages: Vec::new(),
                is_builtin: true,
            },
            TrackPreset {
                id: "track_de_en".to_string(),
                name: "Alle Video + Deutsch/Englisch Audio + Alle Untertitel".to_string(),
                description: "Übernimmt alle Videospuren, deutsche & englische Audiospuren sowie alle Untertitelspuren.".to_string(),
                video_mode: VideoPresetMode::All,
                audio_mode: AudioPresetMode::MatchingLanguagesOnly,
                audio_languages: vec!["deu".into(), "ger".into(), "de".into(), "eng".into(), "en".into()],
                audio_fallback_to_first: true,
                subtitle_mode: SubtitlePresetMode::All,
                subtitle_languages: Vec::new(),
                is_builtin: true,
            },
        ]
    }

    pub fn default_encoding_presets() -> Vec<EncodingPreset> {
        vec![
            EncodingPreset {
                id: "enc_hevc_balanced".to_string(),
                name: "Apple Silicon HEVC Balanced (Empfohlen)".to_string(),
                description: "Hocheffiziente H.265 Kompression via Apple VideoToolbox & Neural Engine. Spart ca. 38% Dateigröße.".to_string(),
                config: EncodingConfig {
                    video_codec: "hevc".to_string(),
                    audio_codec: "aac".to_string(),
                    preset: "balanced".to_string(),
                    crf: 23,
                    use_bitrate: false,
                    video_bitrate_kbps: None,
                    audio_bitrate_kbps: 256,
                    resolution_mode: ResolutionOption::Original,
                    custom_width: None,
                    custom_height: None,
                    fps: None,
                    enable_ai_npu: true,
                    hw_accel: "auto".to_string(),
                    container_override: None,
                },
                is_builtin: true,
            },
            EncodingPreset {
                id: "enc_web_1080p".to_string(),
                name: "Web 1080p HEVC 4500k (Proportionale Höhe)".to_string(),
                description: "Skaliert Zielbreite auf 1080p (1920px) mit automatischer Höhenanpassung und 4.5 Mbps Video-Bitrate.".to_string(),
                config: EncodingConfig {
                    video_codec: "hevc".to_string(),
                    audio_codec: "aac".to_string(),
                    preset: "balanced".to_string(),
                    crf: 22,
                    use_bitrate: true,
                    video_bitrate_kbps: Some(4500),
                    audio_bitrate_kbps: 256,
                    resolution_mode: ResolutionOption::CustomWidth,
                    custom_width: Some(1920),
                    custom_height: None,
                    fps: None,
                    enable_ai_npu: true,
                    hw_accel: "auto".to_string(),
                    container_override: None,
                },
                is_builtin: true,
            },
            EncodingPreset {
                id: "enc_web_720p".to_string(),
                name: "Web 720p H.264 2500k (Proportionale Höhe)".to_string(),
                description: "Skaliert Zielbreite auf 720p (1280px) mit automatischer Höhenanpassung und 2.5 Mbps H.264 Video-Bitrate.".to_string(),
                config: EncodingConfig {
                    video_codec: "h264".to_string(),
                    audio_codec: "aac".to_string(),
                    preset: "balanced".to_string(),
                    crf: 23,
                    use_bitrate: true,
                    video_bitrate_kbps: Some(2500),
                    audio_bitrate_kbps: 192,
                    resolution_mode: ResolutionOption::CustomWidth,
                    custom_width: Some(1280),
                    custom_height: None,
                    fps: None,
                    enable_ai_npu: true,
                    hw_accel: "auto".to_string(),
                    container_override: None,
                },
                is_builtin: true,
            },
            EncodingPreset {
                id: "enc_hevc_ultrafast".to_string(),
                name: "HEVC Ultra-Fast (Maximaler Durchsatz)".to_string(),
                description: "Maximale Verarbeitungsgeschwindigkeit mit NPU Fast-Skip Pruning.".to_string(),
                config: EncodingConfig {
                    video_codec: "hevc".to_string(),
                    audio_codec: "aac".to_string(),
                    preset: "ultra-fast".to_string(),
                    crf: 24,
                    use_bitrate: false,
                    video_bitrate_kbps: None,
                    audio_bitrate_kbps: 192,
                    resolution_mode: ResolutionOption::Original,
                    custom_width: None,
                    custom_height: None,
                    fps: None,
                    enable_ai_npu: true,
                    hw_accel: "auto".to_string(),
                    container_override: None,
                },
                is_builtin: true,
            },
            EncodingPreset {
                id: "enc_stream_copy".to_string(),
                name: "Lossless Stream-Copy (0% Neuberechnung)".to_string(),
                description: "Sekundenschnelles Remuxen ohne Qualitätsverlust. Übernimmt Video- und Audio-Bitströme 1:1.".to_string(),
                config: EncodingConfig {
                    video_codec: "copy".to_string(),
                    audio_codec: "copy".to_string(),
                    preset: "ultra-fast".to_string(),
                    crf: 23,
                    use_bitrate: false,
                    video_bitrate_kbps: None,
                    audio_bitrate_kbps: 256,
                    resolution_mode: ResolutionOption::Original,
                    custom_width: None,
                    custom_height: None,
                    fps: None,
                    enable_ai_npu: false,
                    hw_accel: "cpu".to_string(),
                    container_override: None,
                },
                is_builtin: true,
            },
        ]
    }
}
