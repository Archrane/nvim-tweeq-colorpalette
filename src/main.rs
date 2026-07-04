mod app;
mod color;
mod gesture;
mod protocol;
mod ui;

use app::MyApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // 起動引数を解析して初期のHSVA、アルファ有効フラグ、正規化されたHexを取得
    let (initial_hsva, alpha_enabled, initial_hex) = protocol::parse_initial_color();

    // 最小限のウィンドウを起動するための設定
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([280.0, 660.0])
            .with_min_inner_size([280.0, 420.0])
            .with_decorations(true)
            .with_title("Tweeq Color Picker"),
        ..Default::default()
    };

    eframe::run_native(
        "Tweeq Color Picker",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(MyApp::new(
                initial_hsva,
                alpha_enabled,
                initial_hex,
            )))
        }),
    )
}
