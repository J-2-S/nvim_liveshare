local M = {}

M.setup = function()
   vim.api.nvim_create_augroup("LiveShare", { clear = true })
   -- Working autocmd for when the buffer content is changed in various ways
   vim.api.nvim_create_autocmd({ "TextChanged", "TextChangedI", "TextChangedP", "TextChangedT", "TextYankPost" }, {
      group = "LiveShare",
      callback = function()
         -- local content = vim.api.nvim_buf_get_lines(0, 0, vim.api.nvim_buf_line_count(0), false)
         -- print(table.concat(content, "\n"))
      end,
   })
end

return M
