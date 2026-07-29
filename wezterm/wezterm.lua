local wezterm = require("wezterm")
local themes = require("themes")
local bindings = require("bindings")
local window = require("window")

-- Configuración
local config = wezterm.config_builder()

config.prefer_egl = true
config.normalize_output_to_unicode_nfc = true

config.keys = bindings.keys
config.mouse_bindings = bindings.mouse_bindings
config.font = wezterm.font("JetBrainsMono Nerd Font")
config.colors = themes.sekaide
config.enable_scroll_bar = false
config.hide_tab_bar_if_only_one_tab = true
config.default_cursor_style = "BlinkingBar"

-- configuraciones de ventana
for k, v in pairs(window.decorations) do
	config[k] = v
end

return config
