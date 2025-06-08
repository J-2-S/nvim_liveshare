local M = {}

M.setup = function()
   vim.api.nvim_create_augroup("LiveShare", { clear = true })
   vim.api.nvim_create_autocmd("TextChanged", {
      group = "LiveShare",
      callback = function()

      end,
   })
end

return M
