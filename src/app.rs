use crate::color::Hsva;
use crate::gesture::{apply_tweak_delta, detect_tweak_mode, TweakMode};
use eframe::egui;

/// アプリケーションの状態を表す状態機械
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Idle,
    PopupOpen,
    Dragging,
}

/// アプリケーションのメイン状態を管理する構造体
pub struct MyApp {
    pub current_hsva: Hsva,
    pub alpha_enabled: bool,
    pub initial_hex: String,
    pub app_state: AppState,

    // ドラッグ中・モード変更用の状態保持
    pub initial_hsva_for_tweak: Hsva,
    pub accumulated_delta_for_tweak: egui::Vec2,
    pub last_mode: TweakMode,
}

impl MyApp {
    pub fn new(initial_hsva: Hsva, alpha_enabled: bool, initial_hex: String) -> Self {
        Self {
            current_hsva: initial_hsva,
            alpha_enabled,
            initial_hex,
            app_state: AppState::Idle,
            initial_hsva_for_tweak: initial_hsva,
            accumulated_delta_for_tweak: egui::Vec2::ZERO,
            last_mode: TweakMode::Pad,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // キーボードショートカットの判定

        // 1. キャンセル (Escキー) -> 出力せず非0で終了
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            std::process::exit(1);
        }

        // 2. 確定 (Enterキー - PopupOpen時のみ) -> 最終的なHexを出力して正常終了
        if self.app_state == AppState::PopupOpen && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            let hex = crate::color::hsva2hex(self.current_hsva, self.alpha_enabled);
            println!("{}", hex);
            std::process::exit(0);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 背景やボタンなどUIウィジェットを中央寄せで配置
            ui.vertical_centered(|ui| {
                ui.heading("Tweeq Color Picker");
                ui.add_space(8.0);

                // 状態・モード情報の表示 (ロジック確認用)
                ui.label(format!("State: {:?}", self.app_state));
                ui.label(format!("Active Mode: {:?}", self.last_mode));

                let hex = crate::color::hsva2hex(self.current_hsva, self.alpha_enabled);
                ui.label(format!("Initial Hex: {}", self.initial_hex));
                ui.label(format!("Current Hex: {}", hex));
                ui.label(format!(
                    "HSVA: ({:.2}, {:.2}, {:.2}, {:.2})",
                    self.current_hsva.h,
                    self.current_hsva.s,
                    self.current_hsva.v,
                    self.current_hsva.a
                ));
                ui.add_space(12.0);

                // スウォッチボタン (クリックしてPopupOpenをトグル)
                let swatch_text = if self.app_state == AppState::PopupOpen {
                    "Close Swatch (PopupOpen)"
                } else {
                    "Open Swatch (Idle)"
                };
                let swatch_response = ui.button(swatch_text);

                if swatch_response.clicked() {
                    if self.app_state == AppState::PopupOpen {
                        self.app_state = AppState::Idle;
                    } else if self.app_state == AppState::Idle {
                        self.app_state = AppState::PopupOpen;
                    }
                }

                // ポップアッププレースホルダー
                if self.app_state == AppState::PopupOpen {
                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.label("--- Popup Placeholder ---");
                        ui.label("• Drag anywhere in window to tweak");
                        ui.label("• Click outside of buttons to confirm");
                        ui.label("• Press ENTER to confirm");
                        ui.label("• Press ESC to cancel");
                    });
                }
            });

            // 画面全体のインタラクション検出 (余白部分のクリックや全体でのドラッグ)
            // ボタンの上に重ねず、UI全体のイベントを受け取れるようallocate_rectを利用
            let full_rect_response = ui.allocate_rect(ui.max_rect(), egui::Sense::click_and_drag());

            // ドラッグ開始
            if full_rect_response.drag_started() {
                self.app_state = AppState::Dragging;
                self.initial_hsva_for_tweak = self.current_hsva;
                self.accumulated_delta_for_tweak = egui::Vec2::ZERO;
                self.last_mode = ui.input(|i| detect_tweak_mode(i, self.alpha_enabled));
            }

            // ドラッグ中
            if self.app_state == AppState::Dragging {
                if full_rect_response.dragged() {
                    // 1フレームのマウス移動量を取得して積算
                    let delta = ui.input(|i| i.pointer.delta());
                    self.accumulated_delta_for_tweak += delta;

                    // リアルタイムでキー押下状態から現在の TweakMode を判定
                    let current_mode = ui.input(|i| detect_tweak_mode(i, self.alpha_enabled));

                    if current_mode != self.last_mode {
                        // モード切り替えを検知したら、その瞬間の値を再キャプチャして基準値とする
                        self.initial_hsva_for_tweak = self.current_hsva;
                        self.accumulated_delta_for_tweak = egui::Vec2::ZERO;
                        self.last_mode = current_mode;
                    }

                    // 基準値 + 積算移動量から新しい色を適用
                    self.current_hsva = apply_tweak_delta(
                        self.initial_hsva_for_tweak,
                        current_mode,
                        self.accumulated_delta_for_tweak,
                    );
                }

                // ポインター解放 (Dragging からのリリース ➔ 確定終了)
                if full_rect_response.drag_stopped() {
                    let hex = crate::color::hsva2hex(self.current_hsva, self.alpha_enabled);
                    println!("{}", hex);
                    std::process::exit(0);
                }
            }

            // 余白クリック (PopupOpen時の外側クリック ➔ 確定終了)
            if full_rect_response.clicked() {
                if self.app_state == AppState::PopupOpen {
                    let hex = crate::color::hsva2hex(self.current_hsva, self.alpha_enabled);
                    println!("{}", hex);
                    std::process::exit(0);
                } else if self.app_state == AppState::Idle {
                    // Idle状態で背景をクリックしたときも PopupOpen のトグルが可能
                    self.app_state = AppState::PopupOpen;
                }
            }
        });

        // 描画を強制更新して滑らかな追従を実現する
        ctx.request_repaint();
    }
}
