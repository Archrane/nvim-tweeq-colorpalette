local M = {}

M.options = {
  -- デフォルトのバイナリパス。ユーザーがカスタムパスを設定できるようにする。
  bin_path = nil,
}

function M.get_bin_path()
  if M.options.bin_path and M.options.bin_path ~= "" then
    return M.options.bin_path
  end

  -- デフォルトのパスを解決する
  -- lua/tweeq-colorpalette/config.lua から、プロジェクトルートを求める
  local info = debug.getinfo(1, "S").source:sub(2)
  local plugin_dir = info:match("(.-)/lua/tweeq%-colorpalette/config%.lua$")

  if not plugin_dir then
    -- フォールバック
    plugin_dir = vim.fn.fnamemodify(vim.fn.resolve(vim.fn.expand("<sfile>:p")), ":h:h:h")
  end

  -- OSごとの実行バイナリ拡張子
  local ext = ""
  if vim.fn.has("win32") == 1 then
    ext = ".exe"
  end

  return plugin_dir .. "/target/release/nvim-tweeq-colorpalette" .. ext
end

function M.setup(opts)
  M.options = vim.tbl_deep_extend("force", M.options, opts or {})
end

return M
