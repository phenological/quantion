# quantion-mcp

An MCP server for [Quant·ion](https://github.com/phenological/quantion). It lets apps that support MCP (e.g. Claude Code or Codex) use the Quant·ion functions as tools. It runs locally and reads and writes only the folders given with `--allow`.

## Install

macOS and Linux:

```bash
curl -fsSL https://github.com/phenological/quantion/releases/latest/download/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://github.com/phenological/quantion/releases/latest/download/install.ps1 | iex
```

| | macOS, Linux | Windows |
|---|---|---|
| Install folder | `~/.local/bin` | `%LOCALAPPDATA%\Programs\quantion-mcp` |
| Verification | SHA-256 against `SHA256SUMS.txt` | SHA-256 against `SHA256SUMS.txt` |

Environment variables: `QUANTION_MCP_INSTALL_FOLDER` (install folder), `QUANTION_MCP_VERSION` (release, default `latest`).

## Connect it

Add it to your app as an MCP server with this command:

```bash
~/.local/bin/quantion-mcp --allow /path/to/data
```

On Windows:

```powershell
"$env:LOCALAPPDATA\Programs\quantion-mcp\quantion-mcp.exe" --allow C:\path\to\data
```

`--allow` is the folder the tools may read and write. It is required. Repeat it for more folders. Apps that start the server from a settings file need the full path to the program.

## Safety

- Reads and writes only inside the allowed folders. Links that point outside are refused.
- Never overwrites a file unless `overwrite` is true. Never deletes.
- No network access. URLs are refused. Models and libraries must be local files in an allowed folder:
  - `annotate_features`: `library_positive.ion` and `model.onnx` from [josoriom/cipher](https://github.com/josoriom/cipher/releases).
  - `calibrate` and `predict_rts`: a deepwarp model (`.onnx`), see [josoriom/deepwarp](https://github.com/josoriom/deepwarp).

## Run one tool from the command line

```bash
echo '{"folder": "."}' | ~/.local/bin/quantion-mcp call list_files
```

`~/.local/bin/quantion-mcp tools` prints every tool with its input schema, then the library functions each tool runs.
