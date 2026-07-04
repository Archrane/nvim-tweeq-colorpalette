mod app;
mod color;
mod gesture;
mod protocol;

use app::MyApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // 起動引数を解析して初期のHSVA、アルファ有効フラグ、正規化されたHexを取得
    let (initial_hsva, alpha_enabled, initial_hex) = protocol::parse_initial_color();

    // 最小限のウィンドウを起動するための設定
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([240.0, 240.0])
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
