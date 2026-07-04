use super::{contrast_color, to_color32};
use crate::color::{hsv2rgb, Hsva};
use eframe::egui::{self, pos2, Color32, Mesh, Sense, Shape, Stroke, Ui, Vec2};

/// 2Dパッドの一辺のサイズ(正方形)。
pub const PAD_SIZE: f32 = 200.0;

/// 彩度(横軸)×明度(縦軸)を操作する2Dカラーパッドを描画し、クリック/ドラッグ操作を処理します。
///
/// 4頂点(左上: 白、右上: 現在の色相の純色、左下: 黒、右下: 黒)のメッシュ頂点カラー補間のみで
/// グラデーションを生成するため、テクスチャ生成のコストなしで軽量に描画できます。
pub fn color_pad(ui: &mut Ui, hsva: &mut Hsva) -> egui::Response {
    let desired_size = Vec2::splat(PAD_SIZE);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    // クリック/ドラッグ位置から直接 S, V を絶対値として設定する。
    if let Some(pos) = response.interact_pointer_pos() {
        hsva.s = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        hsva.v = (1.0 - (pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
    }

    if ui.is_rect_visible(rect) {
        let pure_hue = to_color32(hsv2rgb(hsva.h, 1.0, 1.0));

        // 左上: 白 (S=0, V=1) / 右上: 純色 (S=1, V=1) / 左下: 黒 (S=0, V=0) / 右下: 黒 (S=1, V=0)
        let mut mesh = Mesh::default();
        mesh.colored_vertex(rect.left_top(), Color32::WHITE);
        mesh.colored_vertex(rect.right_top(), pure_hue);
        mesh.colored_vertex(rect.left_bottom(), Color32::BLACK);
        mesh.colored_vertex(rect.right_bottom(), Color32::BLACK);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(1, 2, 3);
        ui.painter().add(Shape::mesh(mesh));

        ui.painter()
            .rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke);

        // 現在の S, V 位置に円形のドラッグハンドルを描画する。
        let handle_pos = pos2(
            rect.left() + hsva.s * rect.width(),
            rect.top() + (1.0 - hsva.v) * rect.height(),
        );
        let picked = to_color32(hsv2rgb(hsva.h, hsva.s, hsva.v));
        let ring_color = contrast_color(picked);
        ui.painter().circle_filled(handle_pos, 5.0, picked);
        ui.painter()
            .circle_stroke(handle_pos, 6.0, Stroke::new(2.0, ring_color));
    }

    response
}
