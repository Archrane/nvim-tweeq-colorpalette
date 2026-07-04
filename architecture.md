# Neovim Tweeq-like Color Picker Architecture

## 1. 概要
本プロジェクトは、Neovim(Lua)から呼び出され、Rust(egui)で実装された外部GUIがポップアップし、マウスのドラッグによる高精度なカラーピッキングを提供するハイブリッドプラグインである。
tweeqのカラーパレット機能をNeovimにプラグインとして移植する。

## 2. コンポーネント構成
- **Neovim側 (Lua)**: カーソル下のカラーコードを読み取り、外部バイナリを非同期で実行。
- **GUI側 (Rust/egui)**: マウスのドラッグおよびShift/Alt修飾キーを組み合わせたジェスチャーで色を変更。
- 対応ジェスチャー
    - Click: toggle the color picker popup
    - Drag: tweak the color relativelyShift or H: adjust the hue
      - S / V: adjust the saturation / brightness (value)
      - Alt or A: adjust the alpha
      - R / G / B: adjust the red / green / blue channel 

## 3. 通信プロトコル (標準入出力)
- **起動時**: Neovim -> Rust (引数として現在のHexコード `[#RRGGBB]` を渡す)
- **終了時**: Rust -> Neovim (決定されたHexコードを `stdout` に出力してプロセス終了)

## 4. 実装時のルール
- Rust: 不要な外部依存(Crate)を避け標準ライブラリとeguiに集中すること。ただし、必要であれば使用を検討可能。
- Lua: Neovimの組み込みAPI(vim.system や vim.api.*など)を直接使用し、他のUIプラグイン(nui.nvim 等)に依存しないこと。
