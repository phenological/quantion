stop_install <- function(...) {
  message(...)
  quit(status = 1L)
}

find_platform <- function() {
  os <- tolower(Sys.info()[["sysname"]])
  cpu <- tolower(Sys.info()[["machine"]])
  arm <- grepl("arm64|aarch64", cpu)
  intel <- grepl("x86_64|amd64|x86-64", cpu)
  if (os == "darwin" && arm) return(list(name = "macos-arm64", extension = "dylib"))
  if (os == "darwin" && intel) return(list(name = "macos-x86_64", extension = "dylib"))
  if (os == "linux" && arm) return(list(name = "linux-arm64", extension = "so"))
  if (os == "linux" && intel) return(list(name = "linux-x86_64", extension = "so"))
  if (os == "windows" && intel) return(list(name = "windows-x86_64", extension = "dll"))
  stop_install(sprintf("quantion: no prebuilt library for %s (%s); install cargo to build it from source", os, cpu))
}

get_sha256 <- function(path) {
  if (exists("sha256sum", envir = asNamespace("tools"))) {
    return(unname(get("sha256sum", envir = asNamespace("tools"))(path)))
  }
  for (command in c("sha256sum", "shasum -a 256")) {
    output <- tryCatch(suppressWarnings(system(paste(command, shQuote(path)), intern = TRUE)), error = function(e) character())
    if (length(output)) return(strsplit(trimws(output[1]), "[[:space:]]+")[[1]][1])
  }
  stop_install("quantion: could not compute the SHA256 of the downloaded library")
}

download <- function(url, path) {
  local_folder <- Sys.getenv("QUANTION_PREBUILT_DIR")
  if (nzchar(local_folder)) {
    ok <- file.copy(file.path(local_folder, basename(url)), path, overwrite = TRUE)
    if (!ok) stop_install("quantion: ", basename(url), " is not in QUANTION_PREBUILT_DIR ", local_folder)
    return(invisible())
  }
  ok <- tryCatch(utils::download.file(url, path, mode = "wb", quiet = TRUE) == 0L, error = function(e) FALSE)
  if (!ok) stop_install("quantion: could not download ", url, "; if this version is not released yet, install cargo to build it from source")
}

read_dll_exports <- function(path) {
  bytes <- readBin(path, "raw", file.info(path)$size)
  read_unsigned <- function(offset, size) sum(as.numeric(bytes[offset + seq_len(size)]) * 256^(seq_len(size) - 1))
  header <- read_unsigned(0x3C, 4)
  if (!identical(bytes[header + 1:4], as.raw(c(0x50, 0x45, 0, 0)))) stop("not a PE file")
  optional_header <- header + 24
  if (read_unsigned(optional_header, 2) != 0x20B) stop("not a 64-bit DLL")
  sections <- optional_header + read_unsigned(header + 20, 2)
  section_count <- read_unsigned(header + 6, 2)
  to_offset <- function(rva) {
    for (index in seq_len(section_count) - 1) {
      section <- sections + 40 * index
      start <- read_unsigned(section + 12, 4)
      if (rva >= start && rva < start + read_unsigned(section + 8, 4)) return(rva - start + read_unsigned(section + 20, 4))
    }
    stop("address outside every section")
  }
  read_name <- function(offset) {
    end <- offset
    while (bytes[end + 1] != as.raw(0)) end <- end + 1
    rawToChar(bytes[(offset + 1):end])
  }
  export_rva <- read_unsigned(optional_header + 112, 4)
  if (export_rva == 0) stop("no export table")
  exports <- to_offset(export_rva)
  names <- to_offset(read_unsigned(exports + 32, 4))
  vapply(seq_len(read_unsigned(exports + 24, 4)) - 1, function(index) read_name(to_offset(read_unsigned(names + 4 * index, 4))), "")
}

version <- trimws(read.dcf("DESCRIPTION")[1, "Version"])
platform <- find_platform()
asset <- sprintf("libquantion-%s.%s", platform$name, platform$extension)
base <- sprintf("https://github.com/phenological/quantion/releases/download/v%s", version)
folder <- file.path("src", "prebuilt")
dir.create(folder, recursive = TRUE, showWarnings = FALSE)
library_path <- file.path(folder, paste0("libquantion.", platform$extension))
sums_path <- file.path(folder, "SHA256SUMS.txt")

message("quantion: downloading ", asset, " from release v", version)
download(paste0(base, "/", asset), library_path)
download(paste0(base, "/SHA256SUMS.txt"), sums_path)

sums <- strsplit(trimws(readLines(sums_path, warn = FALSE)), "[[:space:]]+\\*?")
expected <- ""
for (row in sums) if (length(row) >= 2 && basename(row[2]) == asset) expected <- tolower(row[1])
unlink(sums_path)
if (!nzchar(expected)) stop_install("quantion: ", asset, " is not listed in SHA256SUMS.txt")
if (tolower(get_sha256(library_path)) != expected) stop_install("quantion: SHA256 of ", asset, " does not match")

if (platform$extension == "dylib") {
  if (system2("install_name_tool", c("-id", "@rpath/libquantion.dylib", shQuote(library_path))) != 0L) {
    stop_install("quantion: install_name_tool failed")
  }
  if (system2("codesign", c("--force", "--sign", "-", shQuote(library_path))) != 0L) {
    stop_install("quantion: codesign failed")
  }
}

if (platform$extension == "dll") {
  exports <- tryCatch(read_dll_exports(library_path), error = function(e) stop_install("quantion: could not read the exports of ", asset, ": ", conditionMessage(e)))
  writeLines(c("LIBRARY libquantion.dll", "EXPORTS", exports), file.path(folder, "libquantion.def"))
  writeLines(c(
    "PKG_LIBS = -L$(CURDIR)/prebuilt -lquantion",
    "",
    "all: $(SHLIB)",
    "",
    "$(SHLIB): prebuilt/libquantion.dll.a",
    "",
    "prebuilt/libquantion.dll.a: prebuilt/libquantion.def",
    "\t$(DLLTOOL) --dllname libquantion.dll --input-def prebuilt/libquantion.def --output-lib prebuilt/libquantion.dll.a"
  ), file.path("src", "Makevars.win"))
} else {
  rpath <- if (platform$extension == "dylib") "-Wl,-rpath,@loader_path" else "-Wl,-rpath,'$$ORIGIN'"
  writeLines(sprintf("PKG_LIBS = -L$(CURDIR)/prebuilt -lquantion %s", rpath), file.path("src", "Makevars"))
}
message("quantion: using the prebuilt ", asset, "; cargo is not needed")
