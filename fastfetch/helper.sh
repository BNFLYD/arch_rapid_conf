pick_logo() {
  local logos_dir=~/.config/fastfetch/logos
  if [[ -n "$KITTY_WINDOW_ID" ]] || [[ -n "$WEZTERM_PANE" ]]; then
    # Terminal soporta kitty protocol → elige de todos los archivos
    local logos=("$logos_dir"/*)
  else
    # Fallback a solo texto
    local logos=("$logos_dir"/*.txt)
  fi

  local selected="${logos[$RANDOM%${#logos[@]}]}"
  case "${selected,,}" in
  *.png | *.jpg | *.jpeg | *.webp | *.gif | *.bmp)
    chafa --format kitty --symbols solid "$selected"
    ;;
  *)
    cat "$selected"
    ;;
  esac
}
