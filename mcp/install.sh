#!/bin/sh
set -eu

repository="phenological/quantion"
name="quantion-mcp"

stop() {
  echo "$name install: $1" >&2
  exit 1
}

get_platform() {
  case "$(uname -s)" in
    Darwin) system="macos" ;;
    Linux) system="linux" ;;
    *) stop "$(uname -s) is not supported; see https://github.com/$repository/tree/main/mcp" ;;
  esac
  case "$(uname -m)" in
    arm64 | aarch64) machine="arm64" ;;
    x86_64 | amd64) machine="x86_64" ;;
    *) stop "$(uname -m) is not supported" ;;
  esac
  echo "$system-$machine"
}

get_download_url() {
  if [ -n "${QUANTION_MCP_DOWNLOAD_URL:-}" ]; then
    echo "$QUANTION_MCP_DOWNLOAD_URL"
  elif [ "$1" = "latest" ]; then
    echo "https://github.com/$repository/releases/latest/download"
  else
    echo "https://github.com/$repository/releases/download/v${1#v}"
  fi
}

get_checksum() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    stop "sha256sum or shasum is needed to check the download"
  fi
}

main() {
  install_folder="${QUANTION_MCP_INSTALL_FOLDER:-$HOME/.local/bin}"
  version="${QUANTION_MCP_VERSION:-latest}"
  command -v curl >/dev/null 2>&1 || stop "curl is needed"

  asset="$name-$(get_platform)"
  download_url="$(get_download_url "$version")"
  mkdir -p "$install_folder"
  download="$install_folder/.$name.download"
  checksums="$install_folder/.$name.SHA256SUMS"
  trap 'rm -f "$download" "$checksums"' EXIT INT TERM

  echo "Downloading $asset ($version)"
  curl -fsSL "$download_url/$asset" -o "$download" || stop "$asset could not be downloaded from $download_url"
  curl -fsSL "$download_url/SHA256SUMS.txt" -o "$checksums" || stop "SHA256SUMS.txt could not be downloaded"

  expected="$(awk -v asset="$asset" '$2 == asset || $2 == "*" asset { print $1 }' "$checksums")"
  [ -n "$expected" ] || stop "SHA256SUMS.txt has no checksum for $asset"
  [ "$(get_checksum "$download")" = "$expected" ] || stop "the checksum of $asset does not match; nothing was installed"

  chmod 755 "$download"
  "$download" --version >/dev/null 2>&1 || stop "$asset does not run on this computer"
  mv -f "$download" "$install_folder/$name"

  echo "Installed $("$install_folder/$name" --version) in $install_folder"
  echo "Connect it to Claude Code with:"
  echo "  claude mcp add --scope user quantion -- $install_folder/$name --allow /path/to/data"
  echo "Or to Codex with:"
  echo "  codex mcp add quantion -- $install_folder/$name --allow /path/to/data"
}

main "$@"
