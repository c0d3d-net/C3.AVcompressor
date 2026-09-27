use anyhow::{anyhow, Result};

/// Calculate target (width, height) maintaining aspect ratio with even dimensions.
///
/// Handles:
/// - Explicit width (auto-calculates height preserving aspect ratio, even number)
/// - Explicit height (auto-calculates width preserving aspect ratio, even number)
/// - Presets: "4k", "2160p", "1440p", "1080p", "720p", "480p"
/// - Custom expressions: "1920x1080", "1280x-1", "1280xauto", "1280"
pub fn resolve_dimensions(
    orig_w: u32,
    orig_h: u32,
    req_width: Option<u32>,
    req_height: Option<u32>,
    req_res: Option<&str>,
) -> Result<Option<(u32, u32)>> {
    if orig_w == 0 || orig_h == 0 {
        return Err(anyhow!("Original video dimensions cannot be zero"));
    }

    // Case 1: Explicit --width provided
    if let Some(w) = req_width {
        let even_w = make_even(w);
        let aspect = orig_h as f64 / orig_w as f64;
        let mut calc_h = (even_w as f64 * aspect).round() as u32;
        calc_h = make_even(calc_h.max(2));
        return Ok(Some((even_w, calc_h)));
    }

    // Case 2: Explicit --height provided
    if let Some(h) = req_height {
        let even_h = make_even(h);
        let aspect = orig_w as f64 / orig_h as f64;
        let mut calc_w = (even_h as f64 * aspect).round() as u32;
        calc_w = make_even(calc_w.max(2));
        return Ok(Some((calc_w, even_h)));
    }

    // Case 3: Parse --resolution string
    if let Some(res) = req_res {
        let s = res.trim().to_lowercase();
        if s.is_empty() {
            return Ok(None);
        }

        // Standard Presets
        match s.as_str() {
            "4k" | "2160p" | "uhd" => {
                return Ok(Some(calculate_proportional(orig_w, orig_h, Some(3840), None)));
            }
            "1440p" | "2k" | "qhd" => {
                return Ok(Some(calculate_proportional(orig_w, orig_h, None, Some(1440))));
            }
            "1080p" | "fhd" => {
                return Ok(Some(calculate_proportional(orig_w, orig_h, None, Some(1080))));
            }
            "720p" | "hd" => {
                return Ok(Some(calculate_proportional(orig_w, orig_h, None, Some(720))));
            }
            "480p" | "sd" => {
                return Ok(Some(calculate_proportional(orig_w, orig_h, None, Some(480))));
            }
            _ => {}
        }

        // Check if format is "WIDTHxHEIGHT" or "WIDTH:HEIGHT"
        let sep = if s.contains('x') { Some('x') } else if s.contains(':') { Some(':') } else { None };
        if let Some(delimiter) = sep {
            let parts: Vec<&str> = s.split(delimiter).collect();
            if parts.len() == 2 {
                let w_str = parts[0].trim();
                let h_str = parts[1].trim();

                let is_auto_h = matches!(h_str, "-1" | "-2" | "auto" | "?");
                let is_auto_w = matches!(w_str, "-1" | "-2" | "auto" | "?");

                if is_auto_h && !is_auto_w {
                    let w: u32 = w_str.parse().map_err(|_| anyhow!("Invalid width: {}", w_str))?;
                    return Ok(Some(calculate_proportional(orig_w, orig_h, Some(w), None)));
                } else if is_auto_w && !is_auto_h {
                    let h: u32 = h_str.parse().map_err(|_| anyhow!("Invalid height: {}", h_str))?;
                    return Ok(Some(calculate_proportional(orig_w, orig_h, None, Some(h))));
                } else if !is_auto_w && !is_auto_h {
                    let w: u32 = w_str.parse().map_err(|_| anyhow!("Invalid width: {}", w_str))?;
                    let h: u32 = h_str.parse().map_err(|_| anyhow!("Invalid height: {}", h_str))?;
                    return Ok(Some((make_even(w), make_even(h))));
                }
            }
        }

        // Check if single number passed (treat as target width)
        if let Ok(w) = s.parse::<u32>() {
            return Ok(Some(calculate_proportional(orig_w, orig_h, Some(w), None)));
        }

        return Err(anyhow!("Unsupported resolution format: '{}'. Examples: 1920x1080, 1280x-1, 1080p, 1280", res));
    }

    Ok(None)
}

fn calculate_proportional(orig_w: u32, orig_h: u32, target_w: Option<u32>, target_h: Option<u32>) -> (u32, u32) {
    if let Some(w) = target_w {
        let even_w = make_even(w);
        let aspect = orig_h as f64 / orig_w as f64;
        let mut calc_h = (even_w as f64 * aspect).round() as u32;
        calc_h = make_even(calc_h.max(2));
        (even_w, calc_h)
    } else if let Some(h) = target_h {
        let even_h = make_even(h);
        let aspect = orig_w as f64 / orig_h as f64;
        let mut calc_w = (even_h as f64 * aspect).round() as u32;
        calc_w = make_even(calc_w.max(2));
        (calc_w, even_h)
    } else {
        (make_even(orig_w), make_even(orig_h))
    }
}

fn make_even(val: u32) -> u32 {
    if val % 2 != 0 {
        val + 1
    } else {
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_auto_height() {
        // 4K 16:9 input (3840x2160) -> scale to width 1920
        let (w, h) = resolve_dimensions(3840, 2160, Some(1920), None, None).unwrap().unwrap();
        assert_eq!((w, h), (1920, 1080));

        // 4K 16:9 input -> scale to width 1280
        let (w, h) = resolve_dimensions(3840, 2160, Some(1280), None, None).unwrap().unwrap();
        assert_eq!((w, h), (1280, 720));

        // 21:9 ultrawide (3440x1440) -> scale to width 2560
        let (w, h) = resolve_dimensions(3440, 1440, Some(2560), None, None).unwrap().unwrap();
        assert_eq!(w, 2560);
        assert_eq!(h % 2, 0); // must be even
    }

    #[test]
    fn test_height_auto_width() {
        let (w, h) = resolve_dimensions(3840, 2160, None, Some(720), None).unwrap().unwrap();
        assert_eq!((w, h), (1280, 720));
    }

    #[test]
    fn test_resolution_strings() {
        // Preset "1080p"
        let (w, h) = resolve_dimensions(3840, 2160, None, None, Some("1080p")).unwrap().unwrap();
        assert_eq!((w, h), (1920, 1080));

        // "1280x-1" or "1280xauto"
        let (w, h) = resolve_dimensions(1920, 1080, None, None, Some("1280x-1")).unwrap().unwrap();
        assert_eq!((w, h), (1280, 720));

        let (w, h) = resolve_dimensions(1920, 1080, None, None, Some("1280xauto")).unwrap().unwrap();
        assert_eq!((w, h), (1280, 720));

        // Single width number "1280"
        let (w, h) = resolve_dimensions(1920, 1080, None, None, Some("1280")).unwrap().unwrap();
        assert_eq!((w, h), (1280, 720));
    }
}
