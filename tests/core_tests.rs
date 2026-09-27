#[cfg(test)]
mod tests {

    #[test]
    fn test_time_parsing() {
        use c3avcompressor::utils::time::parse_time_to_seconds;

        assert_eq!(parse_time_to_seconds("00:00:10").unwrap(), 10.0);
        assert_eq!(parse_time_to_seconds("01:30").unwrap(), 90.0);
        assert_eq!(parse_time_to_seconds("2h15m").unwrap(), 8100.0);
        assert_eq!(parse_time_to_seconds("500ms").unwrap(), 0.5);
    }

    #[test]
    fn test_npu_detection() {
        use c3avcompressor::npu::detect_npu;

        let npu = detect_npu();
        println!("Detected NPU: {} ({:.1} TOPS)", npu.name, npu.estimated_tops);
        assert!(!npu.name.is_empty());
    }

    #[test]
    fn test_ai_stream_optimizer() {
        use c3avcompressor::npu::AiStreamOptimizer;

        let optimizer = AiStreamOptimizer::new();
        let plan = optimizer.plan_encoding(23, 1920, 1080, 60.0, "balanced");
        assert!(plan.recommended_crf >= 16 && plan.recommended_crf <= 38);
        assert!(plan.estimated_bitrate_reduction_percent > 0.0);
    }

    #[test]
    fn test_bitrate_parsing() {
        use c3avcompressor::utils::bitrate::parse_bitrate_to_kbps;

        assert_eq!(parse_bitrate_to_kbps("5M").unwrap(), 5000);
        assert_eq!(parse_bitrate_to_kbps("2.5m").unwrap(), 2500);
        assert_eq!(parse_bitrate_to_kbps("4500k").unwrap(), 4500);
        assert_eq!(parse_bitrate_to_kbps("320").unwrap(), 320);
    }

    #[test]
    fn test_auto_height_aspect_ratio() {
        use c3avcompressor::utils::resolution::resolve_dimensions;

        // 4K 16:9 input (3840x2160) -> target width 1920 -> auto height 1080
        let (w, h) = resolve_dimensions(3840, 2160, Some(1920), None, None).unwrap().unwrap();
        assert_eq!((w, h), (1920, 1080));

        // 4K 16:9 input -> target width 1280 -> auto height 720
        let (w, h) = resolve_dimensions(3840, 2160, Some(1280), None, None).unwrap().unwrap();
        assert_eq!((w, h), (1280, 720));

        // Ultrawide 21:9 input (3440x1440) -> target width 2560 -> even height preserving ratio
        let (w, h) = resolve_dimensions(3440, 1440, Some(2560), None, None).unwrap().unwrap();
        assert_eq!(w, 2560);
        assert_eq!(h % 2, 0); // Must be even for video codecs
    }

    #[test]
    fn test_transcoding_presets_store() {
        use c3avcompressor::presets::{PresetStore, ResolutionOption};
        use c3avcompressor::utils::resolution::resolve_dimensions;

        let store = PresetStore::load_or_default();
        assert!(!store.encoding_presets.is_empty());

        // Find 1080p Web Preset
        let p_1080p = store.find_encoding_preset("Web 1080p").expect("Web 1080p preset should exist");
        assert_eq!(p_1080p.config.video_codec, "hevc");
        assert_eq!(p_1080p.config.audio_codec, "aac");
        assert_eq!(p_1080p.config.resolution_mode, ResolutionOption::CustomWidth);
        assert_eq!(p_1080p.config.custom_width, Some(1920));
        assert_eq!(p_1080p.config.video_bitrate_kbps, Some(4500));
        assert_eq!(p_1080p.config.audio_bitrate_kbps, 256);

        // Verify proportional height calculation using preset's width (3840x2160 -> 1920x1080)
        let (w, h) = resolve_dimensions(3840, 2160, p_1080p.config.custom_width, None, None).unwrap().unwrap();
        assert_eq!((w, h), (1920, 1080));

        // Find 720p Web Preset
        let p_720p = store.find_encoding_preset("Web 720p").expect("Web 720p preset should exist");
        assert_eq!(p_720p.config.video_codec, "h264");
        assert_eq!(p_720p.config.resolution_mode, ResolutionOption::CustomWidth);
        assert_eq!(p_720p.config.custom_width, Some(1280));
        assert_eq!(p_720p.config.video_bitrate_kbps, Some(2500));

        let (w_720, h_720) = resolve_dimensions(1920, 1080, p_720p.config.custom_width, None, None).unwrap().unwrap();
        assert_eq!((w_720, h_720), (1280, 720));
    }
}
