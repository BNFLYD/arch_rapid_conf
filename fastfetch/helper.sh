pick_logo() {
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd 2>/dev/null)"
  local logos_dir="$script_dir/logos"
  if [[ -n "$KITTY_WINDOW_ID" ]] || [[ -n "$WEZTERM_PANE" ]]; then
    local logos=("$logos_dir"/*.*)
  else
    local logos=("$logos_dir"/*.txt)
  fi

  local selected="${logos[$RANDOM%${#logos[@]}]}"
  ln -sf "$selected" "$logos_dir/current_logo"

  # ─── Generar config.jsonc con el type correcto ───
  if [[ "$selected" == *.txt ]]; then
    local type="auto"
  else
    local type="kitty"
  fi
  sed "s/__LOGO_TYPE__/$type/" "$script_dir/config.template.jsonc" >"$script_dir/config.jsonc"
}
