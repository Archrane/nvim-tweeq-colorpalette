use crate::color::Hsva;
use eframe::egui;

/// ドラッグ中にどの色要素を調整するかを決定するモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakMode {
    Pad, // 2軸（彩度・明度）
    H,   // 色相 (Hue)
    S,   // 彩度 (Saturation)
    V,   // 明度 (Value / Brightness)
    R,   // 赤 (Red)
    G,   // 緑 (Green)
    B,   // 青 (Blue)
    A,   // アルファ (Alpha)
}

/// `egui::InputState` から現在のキー/修飾キーの押下状況を見て `TweakMode` を判定します。
pub fn detect_tweak_mode(input: &egui::InputState, alpha_enabled: bool) -> TweakMode {
    // 優先度：
    // 1. Shift または H
    // 2. S
    // 3. V
    // 4. Alt または A (alpha_enabled 時のみ)
    // 5. R
    // 6. G
    // 7. B
    // 8. なし (Pad)
    if input.modifiers.shift || input.key_down(egui::Key::H) {
        TweakMode::H
    } else if input.key_down(egui::Key::S) {
        TweakMode::S
    } else if input.key_down(egui::Key::V) {
        TweakMode::V
    } else if alpha_enabled && (input.modifiers.alt || input.key_down(egui::Key::A)) {
        TweakMode::A
    } else if input.key_down(egui::Key::R) {
        TweakMode::R
    } else if input.key_down(egui::Key::G) {
        TweakMode::G
    } else if input.key_down(egui::Key::B) {
        TweakMode::B
    } else {
        TweakMode::Pad
    }
}

/// マウスの相対移動量（delta）と現在の `TweakMode` に応じて `Hsva` を更新します。
/// `tweakWidth = 240.0` で正規化を行います。
pub fn apply_tweak_delta(color: Hsva, mode: TweakMode, delta: egui::Vec2) -> Hsva {
    const TWEAK_WIDTH: f32 = 240.0;
    let ndx = delta.x / TWEAK_WIDTH;
    let ndy = -delta.y / TWEAK_WIDTH; // Y軸は上方向が正になるよう反転

    match mode {
        TweakMode::Pad => Hsva {
            s: (color.s + ndx).clamp(0.0, 1.0),
            v: (color.v + ndy).clamp(0.0, 1.0),
            ..color
        },
        TweakMode::H => Hsva {
            h: (color.h + ndx).rem_euclid(1.0),
            ..color
        },
        TweakMode::S => Hsva {
            s: (color.s + ndx).clamp(0.0, 1.0),
            ..color
        },
        TweakMode::V => Hsva {
            v: (color.v + ndy).clamp(0.0, 1.0),
            ..color
        },
        TweakMode::A => Hsva {
            a: (color.a + ndx).clamp(0.0, 1.0),
            ..color
        },
        TweakMode::R | TweakMode::G | TweakMode::B => {
            use crate::color::{hsv2rgb, rgb2hsv_with_fallback};
            let (r, g, b) = hsv2rgb(color.h, color.s, color.v);
            let mut r = r.clamp(0.0, 1.0);
            let mut g = g.clamp(0.0, 1.0);
            let mut b = b.clamp(0.0, 1.0);

            match mode {
                TweakMode::R => r = (r + ndx).clamp(0.0, 1.0),
                TweakMode::G => g = (g + ndx).clamp(0.0, 1.0),
                TweakMode::B => b = (b + ndx).clamp(0.0, 1.0),
                _ => unreachable!(),
            }

            let (new_h, new_s, new_v) = rgb2hsv_with_fallback(r, g, b, color.h);
            Hsva {
                h: new_h,
                s: new_s,
                v: new_v,
                a: color.a,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tweak_mode_default() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            ctx.input(|i| {
                assert_eq!(detect_tweak_mode(i, false), TweakMode::Pad);
            });
        });
    }

    #[test]
    fn test_detect_tweak_mode_keys() {
        // 1. Shift
        {
            let ctx = egui::Context::default();
            let mut raw = egui::RawInput::default();
            raw.modifiers.shift = true;
            let _ = ctx.run(raw, |ctx| {
                ctx.input(|i| {
                    assert_eq!(detect_tweak_mode(i, false), TweakMode::H);
                });
            });
        }

        // 2. Key S
        {
            let ctx = egui::Context::default();
            let mut raw = egui::RawInput::default();
            raw.events.push(egui::Event::Key {
                key: egui::Key::S,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
            let _ = ctx.run(raw, |ctx| {
                ctx.input(|i| {
                    assert_eq!(detect_tweak_mode(i, false), TweakMode::S);
                });
            });
        }

        // 3. Key A with alpha disabled (should fallback to Pad)
        {
            let ctx = egui::Context::default();
            let mut raw = egui::RawInput::default();
            raw.events.push(egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
            let _ = ctx.run(raw.clone(), |ctx| {
                ctx.input(|i| {
                    assert_eq!(detect_tweak_mode(i, false), TweakMode::Pad);
                });
            });

            // 4. Key A with alpha enabled
            let _ = ctx.run(raw, |ctx| {
                ctx.input(|i| {
                    assert_eq!(detect_tweak_mode(i, true), TweakMode::A);
                });
            });
        }
    }

    #[test]
    fn test_apply_tweak_delta_pad() {
        let initial = Hsva {
            h: 0.5,
            s: 0.5,
            v: 0.5,
            a: 1.0,
        };

        // 右上方向 (dx = 24.0, dy = -24.0)
        // Yは -dy なので、ndx = 24.0/240.0 = 0.1, ndy = -(-24.0)/240.0 = 0.1
        let delta = egui::Vec2::new(24.0, -24.0);
        let updated = apply_tweak_delta(initial, TweakMode::Pad, delta);

        assert!((updated.s - 0.6).abs() < 1e-5);
        assert!((updated.v - 0.6).abs() < 1e-5);
        assert_eq!(updated.h, 0.5);
    }

    #[test]
    fn test_apply_tweak_delta_hue_wrap() {
        let initial = Hsva {
            h: 0.95,
            s: 0.5,
            v: 0.5,
            a: 1.0,
        };

        // 右へ 24px (ndx = 0.1)
        // h = 0.95 + 0.1 = 1.05 -> wrap to 0.05
        let delta = egui::Vec2::new(24.0, 0.0);
        let updated = apply_tweak_delta(initial, TweakMode::H, delta);

        assert!((updated.h - 0.05).abs() < 1e-5);
    }

    #[test]
    fn test_apply_tweak_delta_rgb() {
        let initial = Hsva {
            h: 0.0, // Red (#FF0000)
            s: 1.0,
            v: 1.0,
            a: 1.0,
        };

        // 緑(G)を増やす
        // 右へ 24px (ndx = 0.1) -> G が 0.0 から 0.1 になる
        let delta = egui::Vec2::new(24.0, 0.0);
        let updated = apply_tweak_delta(initial, TweakMode::G, delta);

        // Gが少し増えたので、HSVも変化するはず
        assert!(updated.h > 0.0);
        assert_eq!(updated.a, 1.0);
    }
}
