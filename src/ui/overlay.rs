use super::{contrast_color, fade, pad, picker_popup, slider, to_color32, wheel};
use crate::color::{hsv2rgb, Hsva};
use crate::gesture::TweakMode;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Id, LayerId, Mesh, Order, Painter, Pos2, Rect,
    Shape, Stroke, Vec2,
};

/// 非アクティブなグラフィックの不透明度。
const DIM_OPACITY: f32 = 0.1;
/// アクティブなグラフィックの不透明度。
const ACTIVE_OPACITY: f32 = 1.0;
/// ドラッグ開始点(origin)からアイドル状態のグラフィックスタックまでの垂直方向の間隔。
const STACK_GAP: f32 = 10.0;
/// スウォッチの基準サイズ(`architecture.md`/1.5節の `inputHeight` 相当)。
const SWATCH_SIZE: f32 = 24.0;
/// プレビュー円のスウォッチに対する拡大倍率(1.5節)。
const PREVIEW_SCALE: f32 = 1.8;

/// ドラッグ中(`AppState::Dragging`)の視覚フィードバック(Tweakオーバーレイ)を
/// 画面全体を覆う最前面レイヤーに描画します。
///
/// - `origin`: ドラッグ開始位置(画面座標)。すべてのグラフィックの基準点。
/// - `hsva`: 現在の色(リアルタイム反映後)。
/// - `mode`: 現在アクティブな `TweakMode`。
/// - `alpha_enabled`: アルファチャンネル使用フラグ。
pub fn show(ctx: &egui::Context, origin: Pos2, hsva: Hsva, mode: TweakMode, alpha_enabled: bool) {
    let screen_rect = ctx.screen_rect();
    let layer_id = LayerId::new(Order::Foreground, Id::new("tweeq_tweak_overlay"));
    let painter = ctx.layer_painter(layer_id);

    // 画面全体を薄暗くしてオーバーレイモードであることを分かりやすくする。
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(50));

    let pad_active = matches!(mode, TweakMode::Pad | TweakMode::S | TweakMode::V);
    let hue_active = matches!(mode, TweakMode::H);
    let alpha_active = matches!(mode, TweakMode::A);
    let r_active = matches!(mode, TweakMode::R);
    let g_active = matches!(mode, TweakMode::G);
    let b_active = matches!(mode, TweakMode::B);

    let pad_size = pad::PAD_SIZE;
    let width = picker_popup::POPUP_WIDTH;
    let slider_h = slider::SLIDER_HEIGHT;

    // --- アイドル時(非アクティブ時)のスタッキレイアレイアウトを算出する ---
    // origin の少し下に、パッド → 色相環(ホイール) → アルファスライダー → RGBバー(3分割)の順に並べる。
    let mut y = origin.y + 28.0;
    let pad_idle = Rect::from_min_size(pos2(origin.x - pad_size / 2.0, y), Vec2::splat(pad_size));
    y += pad_size + STACK_GAP;
    let wheel_diam = wheel::OUTER_RADIUS * 2.0;
    let hue_idle_center = pos2(origin.x, y + wheel::OUTER_RADIUS);
    y += wheel_diam + STACK_GAP;
    let alpha_idle = if alpha_enabled {
        let r = Rect::from_min_size(pos2(origin.x - width / 2.0, y), vec2(width, slider_h));
        y += slider_h + STACK_GAP;
        Some(r)
    } else {
        None
    };
    let rgb_bar_w = (width - 2.0 * STACK_GAP) / 3.0;
    let r_idle = Rect::from_min_size(pos2(origin.x - width / 2.0, y), vec2(rgb_bar_w, slider_h));
    let g_idle = Rect::from_min_size(
        pos2(r_idle.right() + STACK_GAP, y),
        vec2(rgb_bar_w, slider_h),
    );
    let b_idle = Rect::from_min_size(
        pos2(g_idle.right() + STACK_GAP, y),
        vec2(rgb_bar_w, slider_h),
    );

    // --- 「仮想キャンバスの逆オフセット」: アクティブなグラフィックは、
    // 現在値に対応するハンドル位置がちょうど origin に来るよう配置し直す。
    // これにより、非表示の(ロックされた)カーソル位置を中心に、
    // グラフィック側が逆方向にパンしているような錯覚を生む。
    let pad_rect = if pad_active {
        Rect::from_min_size(
            pos2(
                origin.x - hsva.s * pad_size,
                origin.y - (1.0 - hsva.v) * pad_size,
            ),
            Vec2::splat(pad_size),
        )
    } else {
        pad_idle
    };
    draw_pad(&painter, pad_rect, hsva, opacity_for(pad_active));

    let hue_center = if hue_active {
        pos2(origin.x, origin.y + wheel::INDICATOR_RADIUS)
    } else {
        hue_idle_center
    };
    wheel::draw_hue_wheel(&painter, hue_center, hsva, opacity_for(hue_active));

    if alpha_enabled {
        let alpha_rect = if alpha_active {
            Rect::from_min_size(
                pos2(origin.x - hsva.a * width, origin.y - slider_h / 2.0),
                vec2(width, slider_h),
            )
        } else {
            // alpha_enabled が真の場合、alpha_idle は必ず Some になる。
            alpha_idle.expect("alpha_idle is Some when alpha_enabled")
        };
        draw_alpha_slider(&painter, alpha_rect, hsva, opacity_for(alpha_active));
    }

    let (r, g, b) = hsv2rgb(hsva.h, hsva.s, hsva.v);

    let r_rect = if r_active {
        Rect::from_min_size(
            pos2(origin.x - r * rgb_bar_w, origin.y - slider_h / 2.0),
            vec2(rgb_bar_w, slider_h),
        )
    } else {
        r_idle
    };
    draw_channel_bar(
        &painter,
        r_rect,
        r,
        Color32::from_rgb(255, 64, 64),
        opacity_for(r_active),
    );

    let g_rect = if g_active {
        Rect::from_min_size(
            pos2(origin.x - g * rgb_bar_w, origin.y - slider_h / 2.0),
            vec2(rgb_bar_w, slider_h),
        )
    } else {
        g_idle
    };
    draw_channel_bar(
        &painter,
        g_rect,
        g,
        Color32::from_rgb(64, 220, 64),
        opacity_for(g_active),
    );

    let b_rect = if b_active {
        Rect::from_min_size(
            pos2(origin.x - b * rgb_bar_w, origin.y - slider_h / 2.0),
            vec2(rgb_bar_w, slider_h),
        )
    } else {
        b_idle
    };
    draw_channel_bar(
        &painter,
        b_rect,
        b,
        Color32::from_rgb(80, 140, 255),
        opacity_for(b_active),
    );

    // プレビュー円とツールチップラベルは常に最前面・完全な不透明度で描画する。
    draw_preview_circle(&painter, origin, hsva);
    draw_label(&painter, origin, hsva, mode, alpha_enabled);
}

fn opacity_for(active: bool) -> f32 {
    if active {
        ACTIVE_OPACITY
    } else {
        DIM_OPACITY
    }
}

/// 2Dパッド(彩度×明度)のグラフィックを描画する(`ui/pad.rs` の見た目を手続き的に再現)。
fn draw_pad(painter: &Painter, rect: Rect, hsva: Hsva, opacity: f32) {
    let pure_hue = to_color32(hsv2rgb(hsva.h, 1.0, 1.0));

    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), fade(Color32::WHITE, opacity));
    mesh.colored_vertex(rect.right_top(), fade(pure_hue, opacity));
    mesh.colored_vertex(rect.left_bottom(), fade(Color32::BLACK, opacity));
    mesh.colored_vertex(rect.right_bottom(), fade(Color32::BLACK, opacity));
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(Shape::mesh(mesh));

    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, fade(Color32::GRAY, opacity)));

    let handle_pos = pos2(
        rect.left() + hsva.s * rect.width(),
        rect.top() + (1.0 - hsva.v) * rect.height(),
    );
    let picked = to_color32(hsv2rgb(hsva.h, hsva.s, hsva.v));
    painter.circle_filled(handle_pos, 5.0, fade(picked, opacity));
    painter.circle_stroke(
        handle_pos,
        6.0,
        Stroke::new(2.0, fade(contrast_color(picked), opacity)),
    );
}

/// アルファスライダーのグラフィックを描画する(`ui/slider.rs::alpha_slider` の見た目を再現する)。
fn draw_alpha_slider(painter: &Painter, rect: Rect, hsva: Hsva, opacity: f32) {
    let (r, g, b) = hsv2rgb(hsva.h, hsva.s, hsva.v);
    let base = to_color32((r, g, b));
    let transparent = fade(
        Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 0),
        opacity,
    );
    let opaque = fade(
        Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 255),
        opacity,
    );

    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), transparent);
    mesh.colored_vertex(rect.left_bottom(), transparent);
    mesh.colored_vertex(rect.right_top(), opaque);
    mesh.colored_vertex(rect.right_bottom(), opaque);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(Shape::mesh(mesh));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, fade(Color32::GRAY, opacity)));

    let x = rect.left() + hsva.a * rect.width();
    let handle_pos = pos2(x, rect.center().y);
    let radius = rect.height() * 0.5 - 1.0;
    painter.circle_filled(handle_pos, radius, fade(base, opacity));
    painter.circle_stroke(
        handle_pos,
        radius,
        Stroke::new(2.0, fade(contrast_color(base), opacity)),
    );
}

/// R/G/B 単一チャンネル用の簡易バーグラフィックを描画する。
fn draw_channel_bar(painter: &Painter, rect: Rect, value: f32, base_color: Color32, opacity: f32) {
    painter.rect_filled(rect, 2.0, fade(Color32::from_gray(40), opacity));

    let fill_width = rect.width() * value.clamp(0.0, 1.0);
    let fill_rect = Rect::from_min_size(rect.left_top(), vec2(fill_width, rect.height()));
    painter.rect_filled(fill_rect, 2.0, fade(base_color, opacity));

    painter.rect_stroke(rect, 2.0, Stroke::new(1.0, fade(Color32::GRAY, opacity)));

    let x = rect.left() + value.clamp(0.0, 1.0) * rect.width();
    let handle_pos = pos2(x, rect.center().y);
    let radius = rect.height() * 0.5 - 1.0;
    painter.circle_filled(handle_pos, radius, fade(base_color, opacity));
    painter.circle_stroke(
        handle_pos,
        radius,
        Stroke::new(2.0, fade(contrast_color(base_color), opacity)),
    );
}

/// ドラッグ開始点に追従する、現在色で満たされたプレビュー円(通常スウォッチの1.8倍サイズ)を描画する。
fn draw_preview_circle(painter: &Painter, origin: Pos2, hsva: Hsva) {
    let radius = SWATCH_SIZE * PREVIEW_SCALE / 2.0;
    let (r, g, b) = hsv2rgb(hsva.h, hsva.s, hsva.v);
    let base = to_color32((r, g, b));

    // アルファを可視化するための簡易チェッカーボード(円に外接する正方形の範囲で描く)。
    let checker_rect = Rect::from_center_size(origin, Vec2::splat(radius * 2.0));
    draw_checker_square(painter, checker_rect);

    let color_with_alpha = Color32::from_rgba_unmultiplied(
        base.r(),
        base.g(),
        base.b(),
        (hsva.a * 255.0).round() as u8,
    );
    painter.circle_filled(origin, radius, color_with_alpha);
    painter.circle_stroke(origin, radius, Stroke::new(2.5, contrast_color(base)));
}

/// プレビュー円の背景に使う簡易2x2チェッカーボード。
fn draw_checker_square(painter: &Painter, rect: Rect) {
    let dark = Color32::from_gray(96);
    let light = Color32::from_gray(200);
    let half = rect.width() / 2.0;

    let mut mesh = Mesh::default();
    mesh.add_colored_rect(rect, light);
    for row in 0..2 {
        for col in 0..2 {
            if (row + col) % 2 == 0 {
                continue;
            }
            let x0 = rect.left() + col as f32 * half;
            let y0 = rect.top() + row as f32 * half;
            let cell = Rect::from_min_size(pos2(x0, y0), Vec2::splat(half));
            mesh.add_colored_rect(cell, dark);
        }
    }
    painter.add(Shape::mesh(mesh));
}

/// 現在のモード名と数値を示すツールチップラベルを開始点付近に描画する。
fn draw_label(painter: &Painter, origin: Pos2, hsva: Hsva, mode: TweakMode, alpha_enabled: bool) {
    let text = match mode {
        TweakMode::Pad => format!("Sat: {:.1}%   Val: {:.1}%", hsva.s * 100.0, hsva.v * 100.0),
        TweakMode::H => format!("Hue: {:.1}°", hsva.h * 360.0),
        TweakMode::S => format!("Saturation: {:.1}%", hsva.s * 100.0),
        TweakMode::V => format!("Value: {:.1}%", hsva.v * 100.0),
        TweakMode::A => {
            if alpha_enabled {
                format!("Alpha: {:.1}%", hsva.a * 100.0)
            } else {
                "Alpha: disabled".to_string()
            }
        }
        TweakMode::R | TweakMode::G | TweakMode::B => {
            let (r, g, b) = hsv2rgb(hsva.h, hsva.s, hsva.v);
            let (label, value) = match mode {
                TweakMode::R => ("Red", r),
                TweakMode::G => ("Green", g),
                TweakMode::B => ("Blue", b),
                _ => unreachable!(),
            };
            format!("{}: {}", label, (value * 255.0).round() as i32)
        }
    };

    let text_pos = pos2(
        origin.x,
        origin.y - SWATCH_SIZE * PREVIEW_SCALE / 2.0 - 10.0,
    );

    // 背景に暗い矩形を敷いて可読性を確保する。
    let galley = painter.layout_no_wrap(text.clone(), FontId::proportional(15.0), Color32::WHITE);
    let bg_rect = Rect::from_center_size(
        pos2(text_pos.x, text_pos.y - galley.size().y / 2.0),
        galley.size() + vec2(12.0, 6.0),
    );
    painter.rect_filled(bg_rect, 4.0, Color32::from_black_alpha(200));

    painter.text(
        text_pos,
        Align2::CENTER_BOTTOM,
        text,
        FontId::proportional(15.0),
        Color32::WHITE,
    );
}
