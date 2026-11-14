# Quant·ion

<img src="https://raw.githubusercontent.com/phenological/quantion/main/assets/icon.svg" align="right" width="200" alt="Quantion icon">

[![test](https://github.com/phenological/quantion/actions/workflows/test.yml/badge.svg)](https://github.com/phenological/quantion/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/quantion)](https://crates.io/crates/quantion)
[![PyPI](https://img.shields.io/pypi/v/quantion)](https://pypi.org/project/quantion/)
[![npm](https://img.shields.io/npm/v/quantion)](https://www.npmjs.com/package/quantion)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.22700673.svg)](https://doi.org/10.5281/zenodo.22700673)

**Ionic Toolkit for LCMS data processing.**

A Rust core with bindings for R, Python and JavaScript.

<br clear="right">

## Install

```bash
cargo add quantion
```

## Quick start

Read an mzML file, calculate the EIC of a target and find its peak.

```rust
use quantion::{CalculateEicOptions, GetPeakOptions, calculate_eic, get_peak, parse_mzml};

let mut file = parse_mzml("sample.mzML")?;
let eic = calculate_eic(&mut file, 260.103, CalculateEicOptions { from: 2.0, to: 3.0, ..Default::default() })?;
let peak = get_peak(&eic, 2.383, GetPeakOptions { range: 0.05, ..Default::default() })?;
println!("rt {} intensity {}", peak.rt, peak.intensity);
```

## Other languages

- [R](https://github.com/phenological/quantion/tree/main/wrappers/r)
- [Python](https://github.com/phenological/quantion/tree/main/wrappers/python)
- [JavaScript](https://github.com/phenological/quantion/tree/main/wrappers/js)

## AI assistants

**Web chat** (claude.ai, ChatGPT, Gemini): the full documentation as one text file is at https://phenological.github.io/quantion/docs/v0.2.3.txt.

**Claude Code, Codex, Claude Desktop, Cursor** (MCP): `quantion-mcp` gives the assistant the Quant·ion functions as tools. It runs locally and reads only the folders you allow.

macOS and Linux:

```bash
curl -fsSL https://github.com/phenological/quantion/releases/latest/download/install.sh | sh
```

```bash
claude mcp add --scope user quantion -- ~/.local/bin/quantion-mcp --allow /path/to/data
```

```bash
codex mcp add quantion -- ~/.local/bin/quantion-mcp --allow /path/to/data
```

Windows (PowerShell):

```powershell
curl.exe -fsSL -o "$HOME\quantion-mcp.exe" https://github.com/phenological/quantion/releases/latest/download/quantion-mcp-windows-x86_64.exe
```

```powershell
claude mcp add --scope user quantion '--' "$HOME\quantion-mcp.exe" --allow C:\path\to\data
```

```powershell
codex mcp add quantion '--' "$HOME\quantion-mcp.exe" --allow C:\path\to\data
```

Claude Desktop, Cursor, Windows checksums and the tool list: [mcp](https://github.com/phenological/quantion/tree/main/mcp).

## Documentation

Every function, option and example: [API Documentation](https://phenological.github.io/quantion/)
