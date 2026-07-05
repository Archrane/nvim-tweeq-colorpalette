if vim.g.loaded_tweeq_colorpalette == 1 then
  return
end
vim.g.loaded_tweeq_colorpalette = 1

-- デフォルトのセットアップ。
-- コマンド実行時にオートロードされるように、picker.pick_color を直接呼び出すコマンドを登録します。
vim.api.nvim_create_user_command("TweeqColorPick", function()
  require("tweeq-colorpalette.picker").pick_color()
end, {
  desc = "Pick color under cursor using Tweeq Color Palette",
})
