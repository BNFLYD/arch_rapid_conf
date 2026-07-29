vim.cmd.highlight("clear")
vim.g.colors_name = "sekaide"

local C = {
  none = "none",
  bg = "#0d0d0d",
  fg = "#c5bfae",
  txt = "#dad4c3",
  red = "#fc3e57",
  pink = "#df4d96",
  purple = "#ac82e9",
  blue = "#7777ff",
  cyan = "#99eedd",
  teal = "#47c8c0",
  acc = "#55d1d0",
  gray = "#cfcfcf",
  dark_gray = "#bec8d1",
  dark_bg = "#302f2c",
  tab_bg = "#373b3e",
  prueba = "#1eff1e",
  steel = "#8f56e1",
}

---@param name string
---@param opts vim.api.keyset.highlight
local function hl(name, opts)
  vim.api.nvim_set_hl(0, name, opts)
end

-- Terminal colors
vim.g.terminal_color_0 = "#0d0d0d"
vim.g.terminal_color_1 = "#ff6363"
vim.g.terminal_color_2 = "#fc3e57"
vim.g.terminal_color_3 = "#7777ff"
vim.g.terminal_color_4 = "#ac57ff"
vim.g.terminal_color_5 = "#fc3e57"
vim.g.terminal_color_6 = "#99eedd"
vim.g.terminal_color_7 = "#c5bfae"
vim.g.terminal_color_8 = "#cfcfcf"
vim.g.terminal_color_9 = "#ff6363"
vim.g.terminal_color_10 = "#00ffdd"
vim.g.terminal_color_11 = "#00ffdd"
vim.g.terminal_color_12 = "#00ffdd"
vim.g.terminal_color_13 = "#fc3e57"
vim.g.terminal_color_14 = "#ac57ff"
vim.g.terminal_color_15 = "#c5bfae"

-- Editor
hl("Normal", { fg = C.txt, bg = C.none })
hl("NormalNC", { fg = C.txt, bg = C.none })
hl("NormalSB", { fg = C.txt, bg = C.none })
hl("Cursor", { fg = C.bg, bg = C.teal })
hl("CursorLine", { bg = C.bg })
hl("CursorLineNr", { fg = C.teal, bold = true })
hl("LineNr", { fg = C.dark_gray })
hl("SignColumn", { bg = C.bg })
hl("ColorColumn", { bg = C.dark_bg })
hl("Conceal", { fg = C.gray })
hl("CursorColumn", { bg = C.dark_bg })
hl("CursorIM", { fg = C.bg, bg = C.teal })
hl("Directory", { fg = C.teal })
hl("EndOfBuffer", { fg = C.bg })
hl("ErrorMsg", { fg = C.red, bold = true })
hl("FoldColumn", { fg = C.gray, bg = C.bg })
hl("Folded", { fg = C.gray, bg = C.dark_bg })
hl("IncSearch", { fg = C.bg, bg = C.teal, bold = true })
hl("CurSearch", { fg = C.bg, bg = C.teal, bold = true })
hl("Substitute", { fg = C.bg, bg = C.acc, bold = true })
hl("MatchParen", { fg = C.acc, bold = true, underline = true })
hl("ModeMsg", { fg = C.fg })
hl("MoreMsg", { fg = C.teal })
hl("MsgArea", { fg = C.fg })
hl("MsgSeparator", { fg = C.gray })
hl("NonText", { fg = C.dark_gray })
hl("Pmenu", { fg = C.fg, bg = C.bg })
hl("PmenuSel", { fg = C.bg, bg = C.acc, bold = true })
hl("PmenuThumb", { bg = C.dark_bg })
hl("PmenuSbar", { bg = C.dark_bg })
hl("Question", { fg = C.teal })
hl("QuickFixLine", { fg = C.bg, bg = C.teal, bold = true })
hl("Search", { fg = C.bg, bg = C.teal })
hl("SpecialKey", { fg = C.dark_gray })
hl("SpellBad", { undercurl = true, sp = C.red })
hl("SpellCap", { undercurl = true, sp = C.blue })
hl("SpellLocal", { undercurl = true, sp = C.teal })
hl("SpellRare", { undercurl = true, sp = C.purple })
hl("StatusLine", { fg = C.bg, bg = C.none }) --lualine
hl("StatusLineNC", { fg = C.gray, bg = C.dark_bg })
hl("TabLine", { fg = C.gray, bg = C.dark_bg })
hl("TabLineFill", { bg = C.bg })
hl("TabLineSel", { fg = C.bg, bg = C.acc, bold = true })
hl("Title", { fg = C.cyan, bold = true })
hl("VertSplit", { fg = C.dark_gray, bg = C.bg })
hl("Visual", { bg = C.tab_bg })
hl("VisualNOS", { bg = C.prueba })
hl("WarningMsg", { fg = C.purple })
hl("WildMenu", { fg = C.bg, bg = C.acc, bold = true })
hl("WinSeparator", { fg = C.dark_gray, bg = C.bg })
hl("Whitespace", { fg = C.dark_gray })

-- Float / Borders
hl("NormalFloat", { fg = C.txt, bg = C.none })
hl("FloatBorder", { fg = C.fg, bg = C.none })
hl("FloatTitle", { fg = C.cyan, bg = C.none, bold = true })

-- Syntax
hl("Comment", { fg = C.gray, italic = true })
hl("Constant", { fg = C.cyan })
hl("String", { fg = C.pink })
hl("Character", { fg = C.pink })
hl("Number", { fg = C.pink })
hl("Boolean", { fg = C.cyan, bold = true })
hl("Float", { fg = C.pink })
hl("Identifier", { fg = C.purple })
hl("Function", { fg = C.purple })
hl("Statement", { fg = C.cyan })
hl("Conditional", { fg = C.cyan })
hl("Repeat", { fg = C.cyan })
hl("Label", { fg = C.cyan })
hl("Operator", { fg = C.purple })
hl("Keyword", { fg = C.cyan })
hl("Exception", { fg = C.cyan })
hl("PreProc", { fg = C.red })
hl("Include", { fg = C.cyan })
hl("Define", { fg = C.cyan })
hl("Macro", { fg = C.red })
hl("PreCondit", { fg = C.red })
hl("Type", { fg = C.blue })
hl("StorageClass", { fg = C.cyan })
hl("Structure", { fg = C.blue })
hl("Typedef", { fg = C.blue })
hl("Special", { fg = C.red })
hl("SpecialChar", { fg = C.cyan })
hl("Tag", { fg = C.cyan })
hl("Delimiter", { fg = C.gray })
hl("SpecialComment", { fg = C.gray, italic = true })
hl("Debug", { fg = C.red })
hl("Underlined", { fg = C.pink, underline = true })
hl("Ignore", { fg = C.gray })
hl("Error", { fg = C.red, bg = C.bg })
hl("Todo", { fg = C.bg, bg = C.pink, bold = true })

-- Diff
hl("DiffAdd", { fg = C.cyan, bg = C.bg })
hl("DiffChange", { fg = C.blue, bg = C.bg })
hl("DiffDelete", { fg = C.red, bg = C.bg })
hl("DiffText", { fg = C.cyan, bg = C.bg, bold = true })

-- Diagnostic
hl("DiagnosticError", { fg = C.red })
hl("DiagnosticWarn", { fg = C.purple })
hl("DiagnosticInfo", { fg = C.blue })
hl("DiagnosticHint", { fg = C.teal })
hl("DiagnosticOk", { fg = C.cyan })
hl("DiagnosticUnderlineError", { undercurl = true, sp = C.red })
hl("DiagnosticUnderlineWarn", { undercurl = true, sp = C.purple })
hl("DiagnosticUnderlineInfo", { undercurl = true, sp = C.blue })
hl("DiagnosticUnderlineHint", { undercurl = true, sp = C.teal })
hl("DiagnosticVirtualTextError", { fg = C.red })
hl("DiagnosticVirtualTextWarn", { fg = C.purple })
hl("DiagnosticVirtualTextInfo", { fg = C.blue })
hl("DiagnosticVirtualTextHint", { fg = C.teal })
hl("DiagnosticFloatingError", { fg = C.red })
hl("DiagnosticFloatingWarn", { fg = C.purple })
hl("DiagnosticFloatingInfo", { fg = C.blue })
hl("DiagnosticFloatingHint", { fg = C.teal })
hl("DiagnosticSignError", { fg = C.red })
hl("DiagnosticSignWarn", { fg = C.purple })
hl("DiagnosticSignInfo", { fg = C.blue })
hl("DiagnosticSignHint", { fg = C.teal })

-- LSP Reference
hl("LspReferenceText", { bg = C.dark_bg })
hl("LspReferenceRead", { bg = C.dark_bg })
hl("LspReferenceWrite", { bg = C.dark_bg })
hl("LspSignatureActiveParameter", { fg = C.pink, bold = true, underline = true })
hl("LspFloatWinNormal", { bg = C.dark_bg })
hl("LspFloatWinBorder", { fg = C.dark_gray })

-- Treesitter
hl("@comment", { fg = C.gray, italic = true })
hl("@comment.error", { fg = C.red })
hl("@comment.warning", { fg = C.purple })
hl("@comment.todo", { fg = C.pink, bold = true })
hl("@comment.note", { fg = C.teal })
hl("@constant", { fg = C.pink })
hl("@constant.builtin", { fg = C.red })
hl("@constant.macro", { fg = C.red })
hl("@string", { fg = C.cyan })
hl("@string.documentation", { fg = C.cyan })
hl("@string.regex", { fg = C.pink })
hl("@string.escape", { fg = C.pink })
hl("@string.special", { fg = C.red })
hl("@character", { fg = C.cyan })
hl("@character.special", { fg = C.red })
hl("@number", { fg = C.cyan })
hl("@boolean", { fg = C.pink, bold = true })
hl("@float", { fg = C.cyan })
hl("@annotation", { fg = C.purple })
hl("@attribute", { fg = C.purple })
hl("@namespace", { fg = C.blue })
hl("@function", { fg = C.purple })
hl("@function.builtin", { fg = C.red })
hl("@function.call", { fg = C.purple })
hl("@function.macro", { fg = C.red })
hl("@function.method", { fg = C.purple })
hl("@function.method.call", { fg = C.purple })
hl("@parameter", { fg = C.fg })
hl("@parameter.reference", { fg = C.fg })
hl("@method", { fg = C.purple })
hl("@method.call", { fg = C.purple })
hl("@field", { fg = C.fg })
hl("@property", { fg = C.fg })
hl("@constructor", { fg = C.blue })
hl("@conditional", { fg = C.pink })
hl("@repeat", { fg = C.pink })
hl("@label", { fg = C.teal })
hl("@operator", { fg = C.fg })
hl("@keyword", { fg = C.pink })
hl("@keyword.function", { fg = C.pink })
hl("@keyword.operator", { fg = C.pink })
hl("@keyword.return", { fg = C.pink })
hl("@exception", { fg = C.pink })
hl("@type", { fg = C.blue })
hl("@type.builtin", { fg = C.blue })
hl("@type.definition", { fg = C.blue })
hl("@type.qualifier", { fg = C.pink })
hl("@storageclass", { fg = C.pink })
hl("@include", { fg = C.pink })
hl("@preproc", { fg = C.red })
hl("@tag", { fg = C.pink })
hl("@tag.attribute", { fg = C.cyan })
hl("@tag.delimiter", { fg = C.gray })
hl("@punctuation.delimiter", { fg = C.gray })
hl("@punctuation.bracket", { fg = C.fg })
hl("@punctuation.special", { fg = C.pink })
hl("@variable", { fg = C.fg })
hl("@variable.builtin", { fg = C.red })
hl("@variable.member", { fg = C.fg })
hl("@variable.parameter", { fg = C.fg })

-- LSP Semantic Tokens
hl("@lsp.type.class", { fg = C.blue })
hl("@lsp.type.function", { fg = C.purple })
hl("@lsp.type.method", { fg = C.purple })
hl("@lsp.type.variable", { fg = C.fg })
hl("@lsp.type.parameter", { fg = C.fg })
hl("@lsp.type.property", { fg = C.fg })
hl("@lsp.type.enum", { fg = C.blue })
hl("@lsp.type.enumMember", { fg = C.cyan })
hl("@lsp.type.namespace", { fg = C.blue })
hl("@lsp.type.interface", { fg = C.blue })
hl("@lsp.type.type", { fg = C.blue })
hl("@lsp.type.typeParameter", { fg = C.cyan })
hl("@lsp.type.keyword", { fg = C.pink })
hl("@lsp.type.modifier", { fg = C.pink })
hl("@lsp.type.comment", { fg = C.gray })
hl("@lsp.type.string", { fg = C.cyan })
hl("@lsp.type.number", { fg = C.cyan })
hl("@lsp.type.boolean", { fg = C.pink })

-- Telescope
hl("TelescopeNormal", { fg = C.fg, bg = C.dark_bg })
hl("TelescopeBorder", { fg = C.dark_gray, bg = C.dark_bg })
hl("TelescopePromptNormal", { fg = C.fg, bg = C.tab_bg })
hl("TelescopePromptBorder", { fg = C.dark_gray, bg = C.tab_bg })
hl("TelescopePromptTitle", { fg = C.pink, bg = C.dark_bg, bold = true })
hl("TelescopePreviewTitle", { fg = C.cyan, bg = C.dark_bg, bold = true })
hl("TelescopeResultsTitle", { fg = C.purple, bg = C.dark_bg, bold = true })
hl("TelescopeSelection", { fg = C.bg, bg = C.pink, bold = true })
hl("TelescopeSelectionCaret", { fg = C.cyan })
hl("TelescopeMatching", { fg = C.cyan, bold = true })
hl("TelescopeMultiIcon", { fg = C.cyan })

-- WhichKey
hl("WhichKey", { fg = C.cyan, bold = true })
hl("WhichKeyDesc", { fg = C.txt })
hl("WhichKeyGroup", { fg = C.teal })
hl("WhichKeySeparator", { fg = C.fg })
hl("WhichKeyBorder", { fg = C.fg, bg = C.none })
hl("WhichKeyFloat", { bg = C.dark_bg })

-- Notify
hl("NotifyERRORBorder", { fg = C.fg, bg = C.dark_bg })
hl("NotifyWARNBorder", { fg = C.fg, bg = C.dark_bg })
hl("NotifyINFOBorder", { fg = C.fg, bg = C.dark_bg })
hl("NotifyDEBUGBorder", { fg = C.fg, bg = C.dark_bg })
hl("NotifyTRACEBorder", { fg = C.fg, bg = C.dark_bg })
hl("NotifyERRORIcon", { fg = C.red })
hl("NotifyWARNIcon", { fg = C.purple })
hl("NotifyINFOIcon", { fg = C.blue })
hl("NotifyDEBUGIcon", { fg = C.gray })
hl("NotifyTRACEIcon", { fg = C.teal })
hl("NotifyERRORTitle", { fg = C.red })
hl("NotifyWARNTitle", { fg = C.purple })
hl("NotifyINFOTitle", { fg = C.blue })
hl("NotifyDEBUGTitle", { fg = C.gray })
hl("NotifyTRACETitle", { fg = C.teal })
hl("NotifyBackground", { bg = C.dark_bg })

-- Noice
hl("NoiceCursor", { fg = C.bg, bg = C.teal })

-- Lazy
hl("LazyProgressDone", { fg = C.cyan, bold = true })
hl("LazyProgressTodo", { fg = C.dark_gray })
hl("LazyButton", { fg = C.bg, bg = C.acc })
hl("LazyButtonActive", { fg = C.bg, bg = C.cyan })
hl("LazyReasonPlugin", { fg = C.gray })
hl("LazyReasonStart", { fg = C.cyan })
hl("LazyReasonEvent", { fg = C.blue })
hl("LazyReasonKeys", { fg = C.purple })
hl("LazyReasonFt", { fg = C.teal })
hl("LazyReasonImport", { fg = C.acc })
hl("LazySpecial", { fg = C.acc })
hl("LazyCommit", { fg = C.cyan })
hl("LazyCommitIssue", { fg = C.red })
hl("LazyUrl", { fg = C.teal, underline = true })
hl("LazyDir", { fg = C.blue })
hl("LazyH1", { fg = C.bg, bg = C.acc, bold = true })
hl("LazyH2", { fg = C.acc, bold = true })

-- NvimTree / Neo-tree
hl("NvimTreeNormal", { fg = C.fg, bg = C.bg })
hl("NvimTreeRootFolder", { fg = C.pink, bold = true })
hl("NvimTreeIndentMarker", { fg = C.dark_gray })
hl("NvimTreeFolderIcon", { fg = C.blue })
hl("NvimTreeFileIcon", { fg = C.fg })
hl("NvimTreeExecFile", { fg = C.cyan })
hl("NvimTreeOpenedFile", { fg = C.pink })
hl("NvimTreeSpecialFile", { fg = C.purple })
hl("NvimTreeImageFile", { fg = C.purple })
hl("NvimTreeSymlink", { fg = C.cyan })
hl("NvimTreeFolderName", { fg = C.blue })
hl("NvimTreeEmptyFolderName", { fg = C.gray })
hl("NvimTreeOpenedFolderName", { fg = C.blue })
hl("NvimTreeGitDirty", { fg = C.purple })
hl("NvimTreeGitNew", { fg = C.cyan })
hl("NvimTreeGitDeleted", { fg = C.red })
hl("NvimTreeGitStaged", { fg = C.cyan })
hl("NvimTreeGitMerge", { fg = C.pink })
hl("NvimTreeGitIgnored", { fg = C.gray })
hl("NvimTreeWinSeparator", { fg = C.dark_gray, bg = C.bg })
hl("NvimTreeCursorLine", { bg = C.dark_bg })
hl("NvimTreeStatuslineNc", { fg = C.gray, bg = C.dark_bg })

-- Oil
hl("OilNormal", { fg = C.fg, bg = C.bg })
hl("OilDir", { fg = C.blue })
hl("OilFile", { fg = C.fg })
hl("OilExec", { fg = C.cyan })
hl("OilSymlink", { fg = C.cyan })
hl("OilSpecial", { fg = C.purple })

-- Snacks Dashboard
hl("SnacksDashboardNormal", { fg = C.txt, bg = C.none })
hl("SnacksDashboardHeader", { fg = C.fg, bold = true })
hl("SnacksDashboardKey", { fg = C.cyan, bold = true })
hl("SnacksDashboardDesc", { fg = C.txt })
hl("SnacksDashboardIcon", { fg = C.fg })
hl("SnacksDashboardFooter", { fg = C.gray })
hl("SnacksDashboardSpecial", { fg = C.teal })

-- Lualine
hl("lualine_a_normal", { fg = C.bg, bg = C.pink })
hl("lualine_b_normal", { fg = C.prueba, bg = C.tab_bg })
hl("lualine_c_normal", { fg = C.prueba, bg = C.bg })
hl("lualine_a_insert", { fg = C.bg, bg = C.teal })
hl("lualine_b_insert", { fg = C.prueba, bg = C.tab_bg })
hl("lualine_a_visual", { fg = C.bg, bg = C.purple })
hl("lualine_b_visual", { fg = C.prueba, bg = C.tab_bg })
hl("lualine_a_replace", { fg = C.bg, bg = C.red })
hl("lualine_b_replace", { fg = C.prueba, bg = C.tab_bg })
hl("lualine_a_command", { fg = C.bg, bg = C.blue })
hl("lualine_b_command", { fg = C.prueba, bg = C.tab_bg })

-- Flash
hl("FlashLabel", { fg = C.bg, bg = C.cyan, bold = true })
hl("FlashMatch", { fg = C.bg, bg = C.prueba, bold = true })
hl("FlashCurrent", { fg = C.bg, bg = C.teal, bold = true })

-- Mini
hl("MiniJump", { fg = C.cyan, underline = true })
hl("MiniJump2spot", { fg = C.prueba, bold = true })
hl("MiniStarterCurrent", { fg = C.bg, bg = C.prueba })
hl("MiniStarterFooter", { fg = C.cyan })
hl("MiniStarterHeader", { fg = C.prueba })
hl("MiniStarterInactive", { fg = C.gray })
hl("MiniStarterItem", { fg = C.fg })
hl("MiniStarterSelected", { fg = C.prueba, bold = true })
hl("MiniStarterPrefix", { fg = C.cyan })
hl("MiniStarterSection", { fg = C.purple })
hl("MiniStarterQuery", { fg = C.cyan })
hl("MiniCompletionActiveParameter", { underline = true })
hl("MiniIndentscopeSymbol", { fg = C.dark_gray })
hl("MiniTrailspace", { bg = C.red })
