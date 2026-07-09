pick_logo() {
  local logos_dir=~/.config/fastfetch/logos
  if [[ -n "$KITTY_WINDOW_ID" ]] || [[ -n "$WEZTERM_PANE" ]]; then
    local logos=("$logos_dir"/*.*)
  else
    local logos=("$logos_dir"/*.txt)
  fi

  local selected="${logos[$RANDOM%${#logos[@]}]}"
  ln -sf "$selected" "$logos_dir/current_logo"
}
