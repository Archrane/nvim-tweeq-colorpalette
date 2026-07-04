#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsva {
    pub h: f32, // 0.0..1.0 (Hue)
    pub s: f32, // 0.0..1.0 (Saturation)
    pub v: f32, // 0.0..1.0 (Value / Brightness)
    pub a: f32, // 0.0..1.0 (Alpha)
}

/// ピュア関数：RGB から HSV への変換。
/// 無彩色（diff == 0）の場合は色相 `H` は 0.0 にフォールバックします。
pub fn rgb2hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    rgb2hsv_with_fallback(r, g, b, 0.0)
}

/// ピュア関数：RGB から HSV への変換（フォールバック色相指定付き）。
/// 無彩色（s=0 または v=0）の場合、色相 `H` に `fallback_h` を維持します。
pub fn rgb2hsv_with_fallback(r: f32, g: f32, b: f32, fallback_h: f32) -> (f32, f32, f32) {
    let r = r.clamp(0.0, 1.0);
    let g = g.clamp(0.0, 1.0);
    let b = b.clamp(0.0, 1.0);

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let diff = max - min;

    let v = max;
    let s = if max == 0.0 {
        0.0
    } else {
        diff / max
    };

    let h = if diff == 0.0 {
        fallback_h
    } else {
        let h_calc = if max == r {
            (g - b) / diff
        } else if max == g {
            (b - r) / diff + 2.0
        } else {
            (r - g) / diff + 4.0
        };
        let h_scaled = h_calc / 6.0;
        h_scaled.rem_euclid(1.0)
    };

    (h, s, v)
}

/// ピュア関数：HSV から RGB への変換。
pub fn hsv2rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h.rem_euclid(1.0);
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    if s == 0.0 {
        return (v, v, v);
    }

    let h_temp = h * 6.0;
    let i = h_temp.floor();
    let f = h_temp - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let i = (i as i32).rem_euclid(6);

    match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

/// HSVA 構造体を Hex 文字列へ変換します。
/// `alpha_enabled` が真であれば8桁（#RRGGBBAA）、偽であれば6桁（#RRGGBB）を出力します。
pub fn hsva2hex(hsva: Hsva, alpha_enabled: bool) -> String {
    let (r, g, b) = hsv2rgb(hsva.h, hsva.s, hsva.v);
    let r_val = (r * 255.0).round() as u8;
    let g_val = (g * 255.0).round() as u8;
    let b_val = (b * 255.0).round() as u8;

    if alpha_enabled {
        let a_val = (hsva.a * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}{:02x}", r_val, g_val, b_val, a_val)
    } else {
        format!("#{:02x}{:02x}{:02x}", r_val, g_val, b_val)
    }
}

/// Hex 文字列から HSVA 構造体へのパース。
/// 入力の文字数によって `alpha_enabled` フラグ（8桁なら true、6桁なら false）を判定して返します。
pub fn hex2hsva(hex: &str) -> Result<(Hsva, bool), anyhow::Error> {
    let hex = hex.trim();
    let hex = if hex.starts_with('#') {
        &hex[1..]
    } else {
        hex
    };

    let len = hex.len();
    if len != 6 && len != 8 {
        anyhow::bail!("Invalid hex format length. Expected 6 or 8 chars, got: {}", len);
    }

    let r_val = u8::from_str_radix(&hex[0..2], 16)?;
    let g_val = u8::from_str_radix(&hex[2..4], 16)?;
    let b_val = u8::from_str_radix(&hex[4..6], 16)?;

    let r = r_val as f32 / 255.0;
    let g = g_val as f32 / 255.0;
    let b = b_val as f32 / 255.0;

    let (h, s, v) = rgb2hsv(r, g, b);

    let (a, alpha_enabled) = if len == 8 {
        let a_val = u8::from_str_radix(&hex[6..8], 16)?;
        (a_val as f32 / 255.0, true)
    } else {
        (1.0, false)
    };

    Ok((Hsva { h, s, v, a }, alpha_enabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb2hsv_hsv2rgb_roundtrip() {
        let colors = vec![
            (1.0, 0.0, 0.0), // Red
            (0.0, 1.0, 0.0), // Green
            (0.0, 0.0, 1.0), // Blue
            (1.0, 1.0, 0.0), // Yellow
            (0.0, 1.0, 1.0), // Cyan
            (1.0, 0.0, 1.0), // Magenta
            (0.5, 0.5, 0.5), // Grey (Achromatic)
            (1.0, 1.0, 1.0), // White
            (0.0, 0.0, 0.0), // Black
        ];

        for (r, g, b) in colors {
            let (h, s, v) = rgb2hsv(r, g, b);
            let (r2, g2, b2) = hsv2rgb(h, s, v);

            assert!((r - r2).abs() < 1e-4, "Red mismatch for r: {}, g: {}, b: {} (got {})", r, g, b, r2);
            assert!((g - g2).abs() < 1e-4, "Green mismatch for r: {}, g: {}, b: {} (got {})", r, g, b, g2);
            assert!((b - b2).abs() < 1e-4, "Blue mismatch for r: {}, g: {}, b: {} (got {})", r, g, b, b2);
        }
    }

    #[test]
    fn test_fallback_chromatic() {
        // s=0 の無彩色の場合、rgb2hsv_with_fallback は指定した fallback_h を維持するか
        let (h, s, v) = rgb2hsv_with_fallback(0.5, 0.5, 0.5, 0.35);
        assert_eq!(h, 0.35);
        assert_eq!(s, 0.0);
        assert_eq!(v, 0.5);
    }

    #[test]
    fn test_hex_conversion() {
        // 6桁のパース
        let hex_6 = "#ff5500";
        let (hsva, alpha_enabled) = hex2hsva(hex_6).unwrap();
        assert!(!alpha_enabled);
        assert_eq!(hsva.a, 1.0);

        let formatted_6 = hsva2hex(hsva, alpha_enabled);
        assert_eq!(formatted_6, "#ff5500");

        // 8桁のパース
        let hex_8 = "#ff5500aa";
        let (hsva_8, alpha_enabled_8) = hex2hsva(hex_8).unwrap();
        assert!(alpha_enabled_8);
        assert!((hsva_8.a - 170.0 / 255.0).abs() < 1e-4);

        let formatted_8 = hsva2hex(hsva_8, alpha_enabled_8);
        assert_eq!(formatted_8, "#ff5500aa");
    }

    #[test]
    fn test_invalid_hex() {
        assert!(hex2hsva("#xyz").is_err());
        assert!(hex2hsva("#12345").is_err());
        assert!(hex2hsva("#1234567").is_err());
    }
}
