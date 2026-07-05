use super::{contrast_color, to_color32};
use crate::color::{hsv2rgb, Hsva};
use eframe::egui::{self, pos2, Color32, Mesh, Painter, Rect, Sense, Shape, Stroke, Ui, Vec2};

/// スライダーの高さ(ハンドル込み)。
pub const SLIDER_HEIGHT: f32 = 18.0;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// スライダーの現在値を示す円形ハンドルを描画する共通ヘルパー。
fn draw_handle(painter: &Painter, rect: Rect, t: f32, fill: Color32) {
    let x = lerp(rect.left(), rect.right(), t.clamp(0.0, 1.0));
    let center = pos2(x, rect.center().y);
    let radius = rect.height() * 0.5 - 1.0;
    painter.circle_filled(center, radius, fill);
    painter.circle_stroke(center, radius, Stroke::new(2.0, contrast_color(fill)));
}

/// アルファ(不透明度)スライダーを描画します。
/// `alpha_enabled` が真の時だけ呼び出す想定です。
/// 背景にチェッカーボード模様を描き、その上に現在の色の透明(左)→不透明(右)グラデーションを重ねます。
pub fn alpha_slider(ui: &mut Ui, hsva: &mut Hsva) -> egui::Response {
    let desired_size = Vec2::new(ui.available_width(), SLIDER_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    if let Some(pos) = response.interact_pointer_pos() {
        hsva.a = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    }

    if ui.is_rect_visible(rect) {
        draw_checkerboard(ui.painter(), rect);

        let (r, g, b) = hsv2rgb(hsva.h, hsva.s, hsva.v);
        let base = to_color32((r, g, b));
        let transparent = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 0);
        let opaque = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 255);

        let mut mesh = Mesh::default();
        mesh.colored_vertex(rect.left_top(), transparent);
        mesh.colored_vertex(rect.left_bottom(), transparent);
        mesh.colored_vertex(rect.right_top(), opaque);
        mesh.colored_vertex(rect.right_bottom(), opaque);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(1, 2, 3);
        ui.painter().add(Shape::mesh(mesh));

        ui.painter()
            .rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke);

        draw_handle(ui.painter(), rect, hsva.a, base);
    }

    response
}

/// 半透明表現用のチェッカーボード模様を描画します。
fn draw_checkerboard(painter: &Painter, rect: Rect) {
    let dark = Color32::from_gray(96);
    let light = Color32::from_gray(200);
    let cell = rect.height() / 2.0;

    let mut mesh = Mesh::default();
    mesh.add_colored_rect(rect, light);

    let cols = (rect.width() / cell).ceil() as i32;
    for row in 0..2 {
        for col in 0..cols {
            if (row + col) % 2 == 0 {
                continue;
            }
            let x0 = rect.left() + col as f32 * cell;
            let y0 = rect.top() + row as f32 * cell;
            let cell_rect = Rect::from_min_size(pos2(x0, y0), Vec2::splat(cell)).intersect(rect);
            if cell_rect.is_positive() {
                mesh.add_colored_rect(cell_rect, dark);
            }
        }
    }
    painter.add(Shape::mesh(mesh));
}
