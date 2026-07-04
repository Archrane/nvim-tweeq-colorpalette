mod color;
mod protocol;

use color::Hsva;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // 起動引数を解析して初期のHSVA、アルファ有効フラグ、正規化されたHexを取得
    let (initial_hsva, alpha_enabled, initial_hex) = protocol::parse_initial_color();

    // 最小限のウィンドウを起動するための設定
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([240.0, 240.0])
            .with_decorations(true)
            .with_title("Tweeq Color Picker (MVP)"),
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

struct MyApp {
    initial_hsva: Hsva,
    alpha_enabled: bool,
    initial_hex: String,
}

impl MyApp {
    fn new(initial_hsva: Hsva, alpha_enabled: bool, initial_hex: String) -> Self {
        Self {
            initial_hsva,
            alpha_enabled,
            initial_hex,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Tweeq MVP");
                ui.add_space(10.0);
                ui.label(format!("Hex: {}", self.initial_hex));
                ui.label(format!(
                    "HSVA: ({:.2}, {:.2}, {:.2}, {:.2})",
                    self.initial_hsva.h,
                    self.initial_hsva.s,
                    self.initial_hsva.v,
                    self.initial_hsva.a
                ));
                ui.label(format!("Alpha Enabled: {}", self.alpha_enabled));
                ui.add_space(20.0);
                ui.colored_label(
                    egui::Color32::from_rgb(255, 85, 0),
                    "Click anywhere in this window\nto confirm & output to stdout",
                );
            });

            // ウィンドウ内のクリックを検知して終了
            let response = ui.allocate_rect(ui.max_rect(), egui::Sense::click());
            if response.clicked() {
                println!("{}", self.initial_hex);
                std::process::exit(0);
            }
        });
    }
}
