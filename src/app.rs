use crate::color::Hsva;
use crate::gesture::{apply_tweak_delta, detect_tweak_mode, TweakMode};
use crate::ui::picker_popup::{self, PickerPopupState};
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

    // ポップアップ内フルUI(数値入力欄など)のローカル状態
    pub popup_ui_state: PickerPopupState,
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
            popup_ui_state: PickerPopupState::new(initial_hsva, alpha_enabled),
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
            let mut popup_rect: Option<egui::Rect> = None;
            let mut swatch_rect = egui::Rect::NOTHING;

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
                swatch_rect = swatch_response.rect;

                if swatch_response.clicked() {
                    if self.app_state == AppState::PopupOpen {
                        self.app_state = AppState::Idle;
                    } else if self.app_state == AppState::Idle {
                        self.app_state = AppState::PopupOpen;
                    }
                }

                // ポップアップ内フルUI(2Dパッド/色相スライダー/アルファスライダー/数値入力/プリセット)
                if self.app_state == AppState::PopupOpen {
                    ui.add_space(10.0);
                    let frame_response = egui::Frame::group(ui.style()).show(ui, |ui| {
                        picker_popup::show(
                            ui,
                            &mut self.current_hsva,
                            &mut self.alpha_enabled,
                            &mut self.popup_ui_state,
                        );
                    });
                    popup_rect = Some(frame_response.response.rect);
                }
            });

            if self.app_state == AppState::PopupOpen {
                // ポップアップ及びスウォッチボタン以外の場所でのクリックを「外側クリック」とみなして確定する。
                // ポップアップ内のウィジェット(パッド/スライダー/ボタン等)へのヒットテストを妨げないよう、
                // 画面全体を覆うSenseウィジェットは使わず、生のポインター状態のみを参照する。
                let clicked_outside = ui.input(|i| {
                    i.pointer.primary_clicked()
                        && i.pointer.interact_pos().map_or(false, |pos| {
                            !swatch_rect.contains(pos)
                                && popup_rect.map_or(true, |rect| !rect.contains(pos))
                        })
                });

                if clicked_outside {
                    let hex = crate::color::hsva2hex(self.current_hsva, self.alpha_enabled);
                    println!("{}", hex);
                    std::process::exit(0);
                }
            } else {
                // PopupOpen以外の状態でのみ、画面全体でのドラッグ/クリックを検出する
                // (スウォッチの直接ドラッグ操作、および背景クリックでのポップアップ開閉)
                let full_rect_response =
                    ui.allocate_rect(ui.max_rect(), egui::Sense::click_and_drag());

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

                // Idle状態での背景クリック ➔ ポップアップを開く
                if full_rect_response.clicked() && self.app_state == AppState::Idle {
                    self.app_state = AppState::PopupOpen;
                }
            }
        });

        // 描画を強制更新して滑らかな追従を実現する
        ctx.request_repaint();
    }
}
