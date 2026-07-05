local M = {}

-- カーソル位置を含む、または行内の最も近い #RRGGBB / #RRGGBBAA カラーコードを検出する
function M.find_color()
  local line = vim.api.nvim_get_current_line()
  -- cursor_col は 0-based なので 1-based に変換
  local cursor_col = vim.api.nvim_win_get_cursor(0)[2] + 1

  local candidates = {}
  local search_pos = 1

  while search_pos <= #line do
    -- 1. まず8桁を検索
    local s8, e8 = line:find("#%x%x%x%x%x%x%x%x", search_pos)
    -- 2. 6桁を検索
    local s6, e6 = line:find("#%x%x%x%x%x%x", search_pos)

    if not s8 and not s6 then
      break
    end

    -- どちらが先に見つかったか、または優先度
    local s, e
    if s8 and s6 then
      if s8 <= s6 then
        s, e = s8, e8
      else
        s, e = s6, e6
      end
    elseif s8 then
      s, e = s8, e8
    else
      s, e = s6, e6
    end

    -- 見つかったマッチが、隣接する16進数文字に繋がっていないか検証する
    -- 例: #1122334455 のような10桁の一部としてマッチしてしまっている場合、それは除外すべき
    local next_char = line:sub(e + 1, e + 1)
    local is_valid = true
    if next_char:match("^%x$") then
      is_valid = false
    end

    if is_valid then
      table.insert(candidates, {
        color = line:sub(s, e),
        start_col = s, -- 1-basedバイトインデックス
        end_col = e,   -- 1-basedバイトインデックス
      })
      search_pos = e + 1
    else
      -- 無効な場合は、1文字進めて再検索
      search_pos = s + 1
    end
  end

  if #candidates == 0 then
    return nil
  end

  -- カーソル位置に一番近い、またはカーソルが載っている候補を選ぶ
  local best_cand = nil
  local min_dist = math.huge

  for _, cand in ipairs(candidates) do
    -- カーソルがカラーコードの上（[start_col, end_col]）にある場合
    if cursor_col >= cand.start_col and cursor_col <= cand.end_col then
      return cand
    end

    -- カーソルからの最短距離を計算
    local dist
    if cursor_col < cand.start_col then
      dist = cand.start_col - cursor_col
    else
      dist = cursor_col - cand.end_col
    end

    if dist < min_dist then
      min_dist = dist
      best_cand = cand
    end
  end

  return best_cand
end

return M
