# quantion-mcp

An MCP server that gives AI assistants (Claude Code, Codex, Claude Desktop, Cursor and others) the [Quant·ion](https://github.com/phenological/quantion) functions as tools. It runs locally and reads and writes only the folders given with `--allow`.

## Install

macOS and Linux:

```bash
curl -fsSL https://github.com/phenological/quantion/releases/latest/download/install.sh | sh
```

It downloads the binary for your system, checks it against `SHA256SUMS.txt` and installs it in `~/.local/bin`. Set `QUANTION_MCP_INSTALL_FOLDER` to install somewhere else, or `QUANTION_MCP_VERSION` to pick a version.

Windows (PowerShell, x86_64):

```powershell
curl.exe -fsSL -o "$HOME\quantion-mcp.exe" https://github.com/phenological/quantion/releases/latest/download/quantion-mcp-windows-x86_64.exe
```

This download is not checked automatically. To check it, compare `Get-FileHash "$HOME\quantion-mcp.exe"` with the line in `SHA256SUMS.txt` on the release page.

## Connect it to your assistant

`--allow` is the folder the tools may read and write. It is required. Repeat it for more folders.

Claude Code:

```bash
claude mcp add --scope user quantion -- ~/.local/bin/quantion-mcp --allow /path/to/data
```

```powershell
claude mcp add --scope user quantion '--' "$HOME\quantion-mcp.exe" --allow C:\path\to\data
```

Codex:

```bash
codex mcp add quantion -- ~/.local/bin/quantion-mcp --allow /path/to/data
```

```powershell
codex mcp add quantion '--' "$HOME\quantion-mcp.exe" --allow C:\path\to\data
```

Claude Desktop, Cursor and other clients use a JSON config. Use the full path, because these apps do not read your shell `PATH`.

macOS and Linux (replace `/Users/me` with your home folder):

```json
{
  "mcpServers": {
    "quantion": {
      "command": "/Users/me/.local/bin/quantion-mcp",
      "args": ["--allow", "/path/to/data"]
    }
  }
}
```

Windows:

```json
{
  "mcpServers": {
    "quantion": {
      "command": "C:\\Users\\me\\quantion-mcp.exe",
      "args": ["--allow", "C:\\path\\to\\data"]
    }
  }
}
```

## Tools

Tools have the same names, options and defaults as the library functions.

| Tool | What it does |
|---|---|
| `list_files` | List the .ion and .mzML files in a folder |
| `write_ion` | Save a sample as .ion, for example to convert mzML |
| `write_mzml` | Save a sample as mzML |
| `get_scans` | Read scans by rt, precursor m/z or both |
| `get_scan_metadata` | Read the attributes and params of one scan |
| `calculate_eic` | Calculate the EIC of one m/z and summarize it, or save every point as CSV |
| `find_peaks` | Find every peak in a trace |
| `get_peak` | Find the one peak closest to an rt |
| `fit_peak` | Fit a Gaussian or EMG curve to one peak |
| `draw_peaks` | Simulate a chromatogram from peak curves |
| `find_targets` | Quantify known compounds in one or more samples |
| `find_transitions` | Quantify SRM/MRM transitions in one or more samples |
| `find_features` | Find every feature in one sample (untargeted), with optional deisotoping |
| `match_features` | Find and match features across every sample of a folder |
| `annotate_features` | Name features from their MS2 spectra (experimental) |
| `calibrate` | Turn the rt seen in anchor samples into reference targets |
| `predict_rts` | Predict where each target elutes with a deepwarp model |

Retention times are in minutes. Replies are rounded to keep them short. Use `save` to get full precision as CSV.

## Safety

- Reads and writes only inside the allowed folders. Links that point outside are refused.
- Never overwrites a file unless `overwrite` is true. Never deletes.
- No network access. URLs are refused. Models and libraries must be local files in an allowed folder:
  - `annotate_features`: `library_positive.ion` and `model.onnx` from [josoriom/cipher](https://github.com/josoriom/cipher/releases).
  - `calibrate` and `predict_rts`: a deepwarp model (`.onnx`), see [josoriom/deepwarp](https://github.com/josoriom/deepwarp).

## Run one tool without an assistant

```bash
echo '{"folder": "."}' | ~/.local/bin/quantion-mcp call list_files
```

`~/.local/bin/quantion-mcp tools` prints every tool with its input schema, then the library functions each tool runs.
