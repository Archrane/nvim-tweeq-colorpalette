local M = {}

local config = require("tweeq-colorpalette.config")
local picker = require("tweeq-colorpalette.picker")

function M.setup(opts)
  config.setup(opts)

  -- ユーザーコマンドの登録
  vim.api.nvim_create_user_command("TweeqColorPick", function()
    picker.pick_color()
  end, {
    desc = "Pick color under cursor using Tweeq Color Palette",
  })
end

return M
