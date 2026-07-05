use super::{contrast_color, fade, to_color32};
use crate::color::{hsv2rgb, Hsva};
use eframe::egui::{vec2, Color32, Mesh, Painter, Pos2, Response, Sense, Shape, Stroke, Ui, Vec2};
use std::f32::consts::TAU;

/// 色相環の外半径(px)。
pub const OUTER_RADIUS: f32 = 100.0;
/// 色相環の内半径(px)。
pub const INNER_RADIUS: f32 = 90.0;
/// リングを分割する扇形の数(10度刻み)。
const SEGMENTS: u32 = 36;
/// 現在の色相を示すインジケータを描く半径位置(外周と内周の中間)。
/// オーバーレイでのアクティブ表示時、この半径上の12時位置がドラッグ開始点(origin)に一致するよう配置する。
pub const INDICATOR_RADIUS: f32 = (OUTER_RADIUS + INNER_RADIUS) / 2.0;
/// インジケータの丸ハンドルの半径。
const INDICATOR_HANDLE_RADIUS: f32 = (OUTER_RADIUS - INNER_RADIUS) / 2.0 + 2.0;

/// ポップアップ内に配置する、独立した色相環(Hue Wheel)ウィジェット。
///
/// リング上を直接クリック・ドラッグすると、中心からのマウス座標の角度(`atan2`)を計算し、
/// `hsva.h` の絶対値として双方向にリアルタイム更新します。
pub fn hue_wheel(ui: &mut Ui, hsva: &mut Hsva) -> Response {
    let size = Vec2::splat(OUTER_RADIUS * 2.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let center = rect.center();

    if let Some(pos) = response.interact_pointer_pos() {
        hsva.h = angle_to_hue(pos, center);
    }

    if ui.is_rect_visible(rect) {
        draw_hue_wheel(ui.painter(), center, *hsva, 1.0);
    }

    response
}

/// マウス座標(画面座標)と中心座標から、12時方向(真上、角度 -π/2)を 0.0 とし、
/// 時計回りに増加する hue(0.0..1.0) の絶対値を求めます。
pub fn angle_to_hue(pos: Pos2, center: Pos2) -> f32 {
    let v = pos - center;
    // 真上を0とした時計回りの角度(-π..π)。
    let angle = v.x.atan2(-v.y);
    (angle / TAU).rem_euclid(1.0)
}

/// 色相環(ドーナツ型リング)を `center` を中心に、36分割した扇形メッシュの
/// 頂点カラー補間でシームレスに描画します。
///
/// 現在の色相を示すインジケータは常に12時方向(真上)に固定して描き、
/// リングのグラデーション自体を現在の `hsva.h` の値に応じて逆方向に回転させます
/// (tweeqの「ハンドル位置を開始点(origin)に一致させる」逆オフセット思想の円環版)。
/// これにより、色相が変化すると虹色のリングがクルクルと回転し、
/// 12時位置のインジケータが常に選択中の色を指し示すようになります。
pub fn draw_hue_wheel(painter: &Painter, center: Pos2, hsva: Hsva, opacity: f32) {
    let mut mesh = Mesh::default();
    for i in 0..=SEGMENTS {
        let frac = i as f32 / SEGMENTS as f32;
        let theta = frac * TAU;
        let dir = vec2(theta.sin(), -theta.cos());
        // リングの回転: 12時位置(frac=0)には常に現在の hue が来るよう、全体を hsva.h だけ進める。
        let t = (frac + hsva.h).rem_euclid(1.0);
        let color = fade(to_color32(hsv2rgb(t, 1.0, 1.0)), opacity);
        mesh.colored_vertex(center + dir * OUTER_RADIUS, color);
        mesh.colored_vertex(center + dir * INNER_RADIUS, color);
        if i < SEGMENTS {
            let idx = i * 2;
            mesh.add_triangle(idx, idx + 1, idx + 2);
            mesh.add_triangle(idx + 1, idx + 2, idx + 3);
        }
    }
    painter.add(Shape::mesh(mesh));

    let gray = fade(Color32::GRAY, opacity);
    painter.circle_stroke(center, OUTER_RADIUS, Stroke::new(1.0, gray));
    painter.circle_stroke(center, INNER_RADIUS, Stroke::new(1.0, gray));

    draw_indicator(painter, center, hsva, opacity);
}

/// 12時方向に固定されたインジケータ(現在の色相の純色を示す丸ハンドル)を描画する。
fn draw_indicator(painter: &Painter, center: Pos2, hsva: Hsva, opacity: f32) {
    let pure_hue = to_color32(hsv2rgb(hsva.h, 1.0, 1.0));
    let handle_pos = center + vec2(0.0, -INDICATOR_RADIUS);
    painter.circle_filled(handle_pos, INDICATOR_HANDLE_RADIUS, fade(pure_hue, opacity));
    painter.circle_stroke(
        handle_pos,
        INDICATOR_HANDLE_RADIUS,
        Stroke::new(2.0, fade(contrast_color(pure_hue), opacity)),
    );
}
