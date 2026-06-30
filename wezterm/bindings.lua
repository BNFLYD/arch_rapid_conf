local wezterm = require 'wezterm'

local leader = { key = "q", mods = "ALT", timeout_milliseconds = 2000 }
local keys = {
    -- F11 = fullscreen
    { key = "F11", mods = "NONE", action = wezterm.action.ToggleFullScreen },

    -- Alt+Enter también hace fullscreen
    { key = "Enter", mods = "ALT", action = wezterm.action.ToggleFullScreen },

    -- Pegar con Ctrl+V
    { key = "v", mods = "CTRL", action = wezterm.action.PasteFrom("Clipboard") },

    {
        mods = "LEADER",
        key = "c",
        action = wezterm.action.SpawnTab "CurrentPaneDomain",
    },

     {
        mods = "LEADER",
        key = "x",
        action = wezterm.action.CloseCurrentPane { confirm = true }
    },
}

local mouse_bindings = {
    -- Pegar con clic derecho como en power shell (sin necesidad de Shift)
    {
      event = { Down = { streak = 1, button = "Right" } },
      mods = "NONE",
      action = wezterm.action.PasteFrom("Clipboard")
    },

    -- Arrastrar ventana con Ctrl+click izquierdo
    {
      event = { Down = { streak = 1, button = "Left" } },
      mods = "CTRL",
      action = wezterm.action.StartWindowDrag,
    },
}

return {
    keys = keys,
    mouse_bindings = mouse_bindings,
}
