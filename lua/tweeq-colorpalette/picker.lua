local M = {}

local color_mod = require("tweeq-colorpalette.color")
local config_mod = require("tweeq-colorpalette.config")

function M.pick_color()
  local cand = color_mod.find_color()
  if not cand then
    vim.notify("No color code found under or near cursor", vim.log.levels.WARN, { title = "Tweeq Color Palette" })
    return
  end

  local bin_path = config_mod.get_bin_path()
  if vim.fn.executable(bin_path) == 0 then
    vim.notify("Binary not executable or not found at: " .. bin_path .. "\nPlease build the Rust project or check config.", vim.log.levels.ERROR, { title = "Tweeq Color Palette" })
    return
  end

  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_pos = vim.api.nvim_win_get_cursor(0)
  local row = cursor_pos[1] - 1 -- 0-based

  -- 1-based から 0-based への変換
  -- cand.start_col は 1-based。
  -- cand.end_col も 1-based。
  local start_col = cand.start_col - 1
  local end_col = cand.end_col

  local cmd = { bin_path, cand.color }

  -- 非同期でバイナリを起動
  vim.system(cmd, { text = true }, function(obj)
    vim.schedule(function()
      if obj.code == 0 then
        -- 正常終了時、stdout から改行などをトリミングしてカラーコードを取得
        local new_color = obj.stdout:gsub("%s+", "")
        if new_color ~= "" and new_color:sub(1, 1) == "#" then
          -- 元のバッファがまだ存在し、有効であることを確認
          if vim.api.nvim_buf_is_valid(bufnr) then
            vim.api.nvim_buf_set_text(bufnr, row, start_col, row, end_col, { new_color })
          end
        end
      else
        -- エラー終了時（Escによるキャンセルは exit 1 とする設計になっているので、エラー通知はしないか、特定コード以外のみにする）
        if obj.code ~= 1 and obj.stderr and obj.stderr ~= "" then
          vim.notify("Color picker error: " .. obj.stderr, vim.log.levels.ERROR, { title = "Tweeq Color Palette" })
        end
      end
    end)
  end)
end

return M
