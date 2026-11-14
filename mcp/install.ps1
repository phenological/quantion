function Stop-Install([string] $Message) {
    throw "quantion-mcp install: $Message"
}

function Get-DownloadUrl([string] $Version) {
    if ($env:QUANTION_MCP_DOWNLOAD_URL) {
        return $env:QUANTION_MCP_DOWNLOAD_URL
    }
    if ($Version -eq 'latest') {
        return 'https://github.com/phenological/quantion/releases/latest/download'
    }
    return "https://github.com/phenological/quantion/releases/download/v$($Version.TrimStart('v'))"
}

function Get-ExpectedChecksum([string] $ChecksumFile, [string] $Asset) {
    foreach ($line in Get-Content $ChecksumFile) {
        $parts = $line -split '\s+'
        if ($parts.Count -ge 2 -and ($parts[1] -eq $Asset -or $parts[1] -eq "*$Asset")) {
            return $parts[0].ToLower()
        }
    }
    return $null
}

function Install-QuantionMcp {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    $installFolder = $env:QUANTION_MCP_INSTALL_FOLDER
    if (-not $installFolder) {
        $installFolder = Join-Path $env:LOCALAPPDATA 'Programs\quantion-mcp'
    }
    $version = $env:QUANTION_MCP_VERSION
    if (-not $version) {
        $version = 'latest'
    }

    $asset = 'quantion-mcp-windows-x86_64.exe'
    $downloadUrl = Get-DownloadUrl $version
    New-Item -ItemType Directory -Force -Path $installFolder | Out-Null
    $download = Join-Path $installFolder '.quantion-mcp.download.exe'
    $checksums = Join-Path $installFolder '.quantion-mcp.SHA256SUMS'
    $target = Join-Path $installFolder 'quantion-mcp.exe'

    try {
        Write-Host "Downloading $asset ($version)"
        try {
            Invoke-WebRequest -UseBasicParsing -Uri "$downloadUrl/$asset" -OutFile $download
            Invoke-WebRequest -UseBasicParsing -Uri "$downloadUrl/SHA256SUMS.txt" -OutFile $checksums
        }
        catch {
            Stop-Install "$asset could not be downloaded from $downloadUrl"
        }

        $expected = Get-ExpectedChecksum $checksums $asset
        if (-not $expected) {
            Stop-Install "SHA256SUMS.txt has no checksum for $asset"
        }
        $actual = (Get-FileHash -Algorithm SHA256 -Path $download).Hash.ToLower()
        if ($actual -ne $expected) {
            Stop-Install "the checksum of $asset does not match; nothing was installed"
        }

        try {
            & $download --version | Out-Null
        }
        catch {
            Stop-Install "$asset does not run on this computer"
        }
        if ($LASTEXITCODE -ne 0) {
            Stop-Install "$asset does not run on this computer"
        }
        Move-Item -Force -Path $download -Destination $target
    }
    finally {
        Remove-Item -Force -ErrorAction SilentlyContinue -Path $download, $checksums
    }

    Write-Host "Installed $(& $target --version) in $installFolder"
    Write-Host 'Add it to your app as an MCP server with this command:'
    Write-Host "  `"$target`" --allow C:\path\to\data"
}

Install-QuantionMcp
