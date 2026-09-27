use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareAccelerator {
    VideoToolbox, // Apple Silicon / macOS HW
    Vaapi,        // Linux Intel / AMD VAAPI
    Nvenc,        // NVIDIA NVENC
    Qsv,          // Intel QuickSync Video
    SoftwareCpu,  // Multi-threaded CPU fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEncoderBackend {
    pub accelerator: HardwareAccelerator,
    pub h264_encoder: String,
    pub hevc_encoder: String,
    pub av1_encoder: String,
    pub prores_encoder: String,
    pub hardware_device: Option<String>,
    pub zero_copy_supported: bool,
}

pub fn detect_hardware_encoder() -> VideoEncoderBackend {
    #[cfg(target_os = "macos")]
    {
        VideoEncoderBackend {
            accelerator: HardwareAccelerator::VideoToolbox,
            h264_encoder: "h264_videotoolbox".to_string(),
            hevc_encoder: "hevc_videotoolbox".to_string(),
            av1_encoder: "libsvtav1".to_string(),
            prores_encoder: "prores_videotoolbox".to_string(),
            hardware_device: Some("videotoolbox".to_string()),
            zero_copy_supported: true,
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Check if NVIDIA driver exists
        if std::path::Path::new("/proc/driver/nvidia/version").exists()
            || std::path::Path::new("/dev/nvidiactl").exists()
        {
            return VideoEncoderBackend {
                accelerator: HardwareAccelerator::Nvenc,
                h264_encoder: "h264_nvenc".to_string(),
                hevc_encoder: "hevc_nvenc".to_string(),
                av1_encoder: "av1_nvenc".to_string(),
                prores_encoder: "prores_ks".to_string(),
                hardware_device: Some("cuda".to_string()),
                zero_copy_supported: false,
            };
        }

        // Check for VAAPI (/dev/dri/renderD128)
        if std::path::Path::new("/dev/dri/renderD128").exists() {
            return VideoEncoderBackend {
                accelerator: HardwareAccelerator::Vaapi,
                h264_encoder: "h264_vaapi".to_string(),
                hevc_encoder: "hevc_vaapi".to_string(),
                av1_encoder: "av1_vaapi".to_string(),
                prores_encoder: "prores_ks".to_string(),
                hardware_device: Some("/dev/dri/renderD128".to_string()),
                zero_copy_supported: true,
            };
        }

        // Fallback to CPU
        VideoEncoderBackend {
            accelerator: HardwareAccelerator::SoftwareCpu,
            h264_encoder: "libx264".to_string(),
            hevc_encoder: "libx265".to_string(),
            av1_encoder: "libsvtav1".to_string(),
            prores_encoder: "prores_ks".to_string(),
            hardware_device: None,
            zero_copy_supported: false,
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        VideoEncoderBackend {
            accelerator: HardwareAccelerator::SoftwareCpu,
            h264_encoder: "libx264".to_string(),
            hevc_encoder: "libx265".to_string(),
            av1_encoder: "libsvtav1".to_string(),
            prores_encoder: "prores_ks".to_string(),
            hardware_device: None,
            zero_copy_supported: false,
        }
    }
}
