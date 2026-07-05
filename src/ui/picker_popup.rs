use super::{pad, slider, wheel};
use crate::color::{hex2hsva, hsv2rgb, hsva2hex, rgb2hsv_with_fallback, Hsva};
use eframe::egui::{self, Button, Color32, ComboBox, DragValue, Ui, Vec2};

/// `tweeq` の `popupWidth` 定数(1.5節)に倣ったポップアップ全体の幅。
pub const POPUP_WIDTH: f32 = 240.0;

/// 数値入力行で選択できる表示モード。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueMode {
    Rgb,
    Hsv,
    Hex,
}

impl ValueMode {
    fn label(self) -> &'static str {
        match self {
            ValueMode::Rgb => "RGB",
            ValueMode::Hsv => "HSV",
            ValueMode::Hex => "HEX",
        }
    }
}

/// ポップアップ内の数値入力行が保持する必要のあるUIローカル状態。
/// (色そのものは呼び出し元が持つ `Hsva`/`alpha_enabled` を単一の真実源とする)
pub struct PickerPopupState {
    value_mode: ValueMode,
    hex_buffer: String,
}

impl PickerPopupState {
    pub fn new(hsva: Hsva, alpha_enabled: bool) -> Self {
        Self {
            value_mode: ValueMode::Hex,
            hex_buffer: hex_digits(hsva, alpha_enabled),
        }
    }
}

/// 先頭の `#` を含まない Hex 文字列を返す(入力欄には別途 `#` ラベルを表示するため)。
fn hex_digits(hsva: Hsva, alpha_enabled: bool) -> String {
    hsva2hex(hsva, alpha_enabled)
        .trim_start_matches('#')
        .to_string()
}

/// 代表的なプリセットカラー(R, G, B / 0..=255)。
const PRESETS: &[(u8, u8, u8)] = &[
    (0xff, 0xff, 0xff), // White
    (0x00, 0x00, 0x00), // Black
    (0xff, 0x3b, 0x30), // Red
    (0xff, 0x95, 0x00), // Orange
    (0xff, 0xcc, 0x00), // Yellow
    (0x34, 0xc7, 0x59), // Green
    (0x30, 0xd5, 0xc8), // Teal
    (0x00, 0x7a, 0xff), // Blue
    (0xaf, 0x52, 0xde), // Purple
    (0xff, 0x2d, 0x55), // Pink
];

/// ポップアップ内フルUI(2Dパッド・色相スライダー・アルファスライダー・数値入力・プリセット)を描画します。
/// `hsva` と `alpha_enabled` は双方向に結線され、UI操作に応じてその場で更新されます。
pub fn show(ui: &mut Ui, hsva: &mut Hsva, alpha_enabled: &mut bool, state: &mut PickerPopupState) {
    ui.set_width(POPUP_WIDTH);
    ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);

    // 1. 2Dカラーパッド(彩度×明度)
    ui.vertical_centered(|ui| {
        pad::color_pad(ui, hsva);
    });

    // 2. 色相環(Hue Wheel)
    ui.vertical_centered(|ui| {
        wheel::hue_wheel(ui, hsva);
    });

    // 3. アルファスライダー(alpha_enabled が true の時のみ表示) + 使用フラグ切替トグル
    ui.horizontal(|ui| {
        ui.checkbox(alpha_enabled, "A");
        if *alpha_enabled {
            slider::alpha_slider(ui, hsva);
        } else {
            ui.weak("Alpha disabled");
        }
    });

    ui.separator();

    // 4. 数値入力行: RGB/HSV/HEX 切り替えドロップダウン + 各チャンネルの数値入力
    ui.horizontal(|ui| {
        ui.label("Mode:");
        ComboBox::from_id_salt("value_mode_combo")
            .selected_text(state.value_mode.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.value_mode, ValueMode::Rgb, "RGB");
                ui.selectable_value(&mut state.value_mode, ValueMode::Hsv, "HSV");
                ui.selectable_value(&mut state.value_mode, ValueMode::Hex, "HEX");
            });
    });

    ui.horizontal(|ui| match state.value_mode {
        ValueMode::Rgb => show_rgb_inputs(ui, hsva),
        ValueMode::Hsv => show_hsv_inputs(ui, hsva),
        ValueMode::Hex => show_hex_input(ui, hsva, *alpha_enabled, state),
    });

    ui.separator();

    // 5. プリセット行
    ui.label("Presets:");
    ui.horizontal_wrapped(|ui| {
        for &(r, g, b) in PRESETS {
            let color = Color32::from_rgb(r, g, b);
            let button = Button::new("").fill(color).min_size(Vec2::splat(20.0));
            if ui
                .add(button)
                .on_hover_text(format!("#{:02x}{:02x}{:02x}", r, g, b))
                .clicked()
            {
                let (h, s, v) = rgb2hsv_with_fallback(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    hsva.h,
                );
                hsva.h = h;
                hsva.s = s;
                hsva.v = v;
            }
        }
    });
}

/// ドラッグでスクラブ調整可能な RGB (0..=255) 数値入力ボックスを描画する。
fn show_rgb_inputs(ui: &mut Ui, hsva: &mut Hsva) {
    let (r, g, b) = hsv2rgb(hsva.h, hsva.s, hsva.v);
    let mut r255 = (r * 255.0).round() as i32;
    let mut g255 = (g * 255.0).round() as i32;
    let mut b255 = (b * 255.0).round() as i32;

    let r_resp = ui.add(DragValue::new(&mut r255).range(0..=255).prefix("R "));
    let g_resp = ui.add(DragValue::new(&mut g255).range(0..=255).prefix("G "));
    let b_resp = ui.add(DragValue::new(&mut b255).range(0..=255).prefix("B "));

    if r_resp.changed() || g_resp.changed() || b_resp.changed() {
        let (h, s, v) = rgb2hsv_with_fallback(
            r255 as f32 / 255.0,
            g255 as f32 / 255.0,
            b255 as f32 / 255.0,
            hsva.h,
        );
        hsva.h = h;
        hsva.s = s;
        hsva.v = v;
    }
}

/// ドラッグでスクラブ調整可能な HSV (H: 度数 / S,V: パーセント) 数値入力ボックスを描画する。
fn show_hsv_inputs(ui: &mut Ui, hsva: &mut Hsva) {
    let mut h_deg = hsva.h * 360.0;
    let mut s_pct = hsva.s * 100.0;
    let mut v_pct = hsva.v * 100.0;

    let h_resp = ui.add(
        DragValue::new(&mut h_deg)
            .range(0.0..=360.0)
            .suffix("°")
            .prefix("H "),
    );
    let s_resp = ui.add(
        DragValue::new(&mut s_pct)
            .range(0.0..=100.0)
            .suffix("%")
            .prefix("S "),
    );
    let v_resp = ui.add(
        DragValue::new(&mut v_pct)
            .range(0.0..=100.0)
            .suffix("%")
            .prefix("V "),
    );

    if h_resp.changed() {
        hsva.h = (h_deg / 360.0).rem_euclid(1.0);
    }
    if s_resp.changed() {
        hsva.s = (s_pct / 100.0).clamp(0.0, 1.0);
    }
    if v_resp.changed() {
        hsva.v = (v_pct / 100.0).clamp(0.0, 1.0);
    }
}

/// HEX文字列の編集ボックスを描画する。フォーカスが外れている間だけ現在色から再同期する。
fn show_hex_input(ui: &mut Ui, hsva: &mut Hsva, alpha_enabled: bool, state: &mut PickerPopupState) {
    ui.label("#");
    let response = ui.add(
        egui::TextEdit::singleline(&mut state.hex_buffer)
            .desired_width(120.0)
            .hint_text("RRGGBB"),
    );

    if response.changed() {
        if let Ok((parsed, _)) = hex2hsva(&state.hex_buffer) {
            hsva.h = parsed.h;
            hsva.s = parsed.s;
            hsva.v = parsed.v;
            if alpha_enabled {
                hsva.a = parsed.a;
            }
        }
    }

    // 編集中でなければ、常に現在の色から正規化された文字列を再表示する。
    if !response.has_focus() {
        state.hex_buffer = hex_digits(*hsva, alpha_enabled);
    }
}
