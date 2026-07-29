#!/usr/bin/env bash
# Cava wrapper — Right channel only
set -o pipefail

CAVA_CONFIG="${HOME}/.config/cava/cava-waybar-right.conf"

# Unicode bar characters from lowest to highest (0-8)
BAR_CHARS=(" " "▁" "▂" "▃" "▄" "▅" "▆" "▇" "█")

# Small delay to let pipewire/pulse be ready before cava starts
sleep 1

# Run cava and convert each line of raw numbers to bars
cava -p "$CAVA_CONFIG" 2>/dev/null | while IFS=' ' read -ra values; do
  line=""
  for val in "${values[@]}"; do
    # Ensure val is within 0-8 range
    if [[ "$val" -ge 0 && "$val" -le 8 ]]; then
      line+="${BAR_CHARS[$val]}"
    fi
  done
  echo "$line"
done
