pub mod overlay;
pub mod pad;
pub mod picker_popup;
pub mod slider;
pub mod wheel;

use eframe::egui::Color32;

/// (r, g, b) の 0.0..=1.0 浮動小数点タプルを `Color32` に変換します。
pub(crate) fn to_color32((r, g, b): (f32, f32, f32)) -> Color32 {
    Color32::from_rgb(
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

/// 背景色に対して視認性の高いコントラストカラー(白 or 黒)を返します。
/// ドラッグハンドルの輪郭線などに使用します。
pub(crate) fn contrast_color(color: Color32) -> Color32 {
    let luminance = 0.299 * color.r() as f32 + 0.587 * color.g() as f32 + 0.114 * color.b() as f32;
    if luminance < 140.0 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

/// 色の不透明度を `opacity`(0.0..=1.0)倍して返します。
/// Tweakオーバーレイで非アクティブなグラフィックを薄く表示する際に使用します。
pub(crate) fn fade(color: Color32, opacity: f32) -> Color32 {
    let a = (color.a() as f32 * opacity).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}
