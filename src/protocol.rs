use crate::color::{hex2hsva, Hsva};
use std::env;

/// 起動引数から初期色をパースします。
/// 引数が存在しない、またはパースに失敗した場合は、
/// デフォルトの白 (#FFFFFF) と alpha_enabled = false を返します。
/// 戻り値: (Hsva構造体, alpha_enabledフラグ, 正規化されたHex文字列)
pub fn parse_initial_color() -> (Hsva, bool, String) {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // 引数省略時は、デフォルト値
        return get_default();
    }

    let input_hex = &args[1];
    match hex2hsva(input_hex) {
        Ok((hsva, alpha_enabled)) => {
            let normalized_hex = if input_hex.starts_with('#') {
                input_hex.clone()
            } else {
                format!("#{}", input_hex)
            };
            (hsva, alpha_enabled, normalized_hex)
        }
        Err(_) => {
            // パース失敗時もデフォルト値
            get_default()
        }
    }
}

fn get_default() -> (Hsva, bool, String) {
    let default_hsva = Hsva {
        h: 0.0,
        s: 0.0,
        v: 1.0,
        a: 1.0,
    };
    (default_hsva, false, "#ffffff".to_string())
}
