-- bootstrap lazy.nvim, LazyVim and your plugins
require("config.lazy")

vim.opt.clipboard:append("unnamedplus")

-- vim.g.clipboard = {
--  name = "OSC 52",
--  copy = {
--    ["+"] = function(lines)
--      vim.api.nvim_call_function("system", { "printf", "\\033]52;+;%s\\007", vim.fn.join(vim.fn.map(lines, "encode64")) })
--    end,
--    ["*"] = function(lines)
--      vim.api.nvim_call_function("system", { "printf", "\\033]52;*;%s\\007", vim.fn.join(vim.fn.map(lines, "encode64")) })
--    end,
--  },
--  paste = {
--    ["+"] = function()
-- Leer del portapapeles es responsabilidad de WezTerm
--    end,
--    ["*"] = function()
--    end,
--  },
--}
