use super::types::{NpuArchitecture, NpuBackend, NpuDeviceInfo};
use std::process::Command;

/// Detect presence and specs of an NPU on the current system (macOS or Linux).
pub fn detect_npu() -> NpuDeviceInfo {
    #[cfg(target_os = "macos")]
    {
        detect_macos_npu()
    }

    #[cfg(target_os = "linux")]
    {
        detect_linux_npu()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        generic_fallback_npu()
    }
}

#[cfg(target_os = "macos")]
fn detect_macos_npu() -> NpuDeviceInfo {
    // Check if running on Apple Silicon
    let arch = std::env::consts::ARCH;
    if arch == "aarch64" {
        // Query CPU / SoC model
        let model = get_sysctl("machdep.cpu.brand_string")
            .or_else(|| get_sysctl("hw.model"))
            .unwrap_or_else(|| "Apple Silicon".to_string());

        let (tops, name) = if model.contains("M4") {
            (38.0, "Apple M4 16-Core Neural Engine (ANE)")
        } else if model.contains("M3") {
            (18.0, "Apple M3 16-Core Neural Engine (ANE)")
        } else if model.contains("M2") {
            (15.8, "Apple M2 16-Core Neural Engine (ANE)")
        } else if model.contains("M1") {
            (11.0, "Apple M1 16-Core Neural Engine (ANE)")
        } else {
            (25.0, "Apple Silicon Neural Engine (ANE)")
        };

        NpuDeviceInfo {
            name: name.to_string(),
            architecture: NpuArchitecture::AppleNeuralEngine,
            backend: NpuBackend::CoreML,
            estimated_tops: tops,
            is_available: true,
            driver_info: format!("macOS CoreML & Metal Acceleration (Mach Model: {})", model),
            supported_precisions: vec!["FP16".into(), "INT8".into(), "BF16".into()],
            zero_copy_unified_memory: true,
        }
    } else {
        // Intel Mac with T2 or CPU SIMD
        NpuDeviceInfo {
            name: "Apple/Intel Core Vector Processing (AVX2)".to_string(),
            architecture: NpuArchitecture::GenericComputeFallback,
            backend: NpuBackend::AccelerateCpu,
            estimated_tops: 2.0,
            is_available: false,
            driver_info: "Intel Mac (No dedicated ANE)".to_string(),
            supported_precisions: vec!["FP32".into(), "FP16".into()],
            zero_copy_unified_memory: false,
        }
    }
}

#[cfg(target_os = "macos")]
fn get_sysctl(key: &str) -> Option<String> {
    let output = Command::new("sysctl").arg("-n").arg(key).output().ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn detect_linux_npu() -> NpuDeviceInfo {
    use std::path::Path;

    // Check modern Linux kernel accel subsystem (/sys/class/accel/)
    // Intel NPU driver: ivpu -> /sys/class/accel/accel0/device/driver
    // AMD Ryzen AI driver: amd_aie
    if Path::new("/sys/class/accel").exists() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/accel") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let driver_path = entry.path().join("device/driver");
                if let Ok(link) = std::fs::read_link(&driver_path) {
                    let driver_name = link.file_name().unwrap_or_default().to_string_lossy();
                    if driver_name.contains("intel_vpu") || driver_name.contains("ivpu") {
                        return NpuDeviceInfo {
                            name: format!("Intel NPU Accelerate Device ({})", name),
                            architecture: NpuArchitecture::IntelNPU,
                            backend: NpuBackend::OpenVINO,
                            estimated_tops: 11.0, // Meteor Lake ~11 TOPS, Lunar Lake ~48 TOPS
                            is_available: true,
                            driver_info: format!("Linux IVPU Driver (/sys/class/accel/{})", name),
                            supported_precisions: vec!["INT8".into(), "FP16".into()],
                            zero_copy_unified_memory: true,
                        };
                    } else if driver_name.contains("amd_aie") || driver_name.contains("aie") {
                        return NpuDeviceInfo {
                            name: format!("AMD Ryzen AI NPU (XDNA) ({})", name),
                            architecture: NpuArchitecture::AmdRyzenAI,
                            backend: NpuBackend::OnnxRuntime,
                            estimated_tops: 16.0, // XDNA1 ~16 TOPS, XDNA2 ~50 TOPS
                            is_available: true,
                            driver_info: format!("Linux AMD AIE Driver (/sys/class/accel/{})", name),
                            supported_precisions: vec!["INT8".into(), "FP16".into(), "BF16".into()],
                            zero_copy_unified_memory: true,
                        };
                    }
                }
            }
        }
    }

    // Check CPU features from /proc/cpuinfo for Intel/AMD NPU or AMX / VNNI
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        if cpuinfo.contains("avx512_vnni") || cpuinfo.contains("amx_tile") {
            return NpuDeviceInfo {
                name: "Host CPU VNNI/AMX Vector Neural Acceleration".to_string(),
                architecture: NpuArchitecture::GenericComputeFallback,
                backend: NpuBackend::OpenVINO,
                estimated_tops: 4.5,
                is_available: true,
                driver_info: "Linux CPU Neural Acceleration (VNNI/AMX)".to_string(),
                supported_precisions: vec!["INT8".into(), "BF16".into(), "FP32".into()],
                zero_copy_unified_memory: true,
            };
        }
    }

    generic_fallback_npu()
}

#[allow(dead_code)]
fn generic_fallback_npu() -> NpuDeviceInfo {
    NpuDeviceInfo {
        name: "Standard CPU Stream Pipeline (No NPU detected)".to_string(),
        architecture: NpuArchitecture::GenericComputeFallback,
        backend: NpuBackend::AccelerateCpu,
        estimated_tops: 1.0,
        is_available: false,
        driver_info: "Generic Software Fallback".to_string(),
        supported_precisions: vec!["FP32".into()],
        zero_copy_unified_memory: false,
    }
}
