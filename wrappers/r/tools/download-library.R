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
  ok <- tryCatch(utils::download.file(url, path, mode = "wb", quiet = TRUE) == 0L, error = function(e) FALSE)
  if (!ok) stop_install("quantion: could not download ", url)
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

rpath <- switch(platform$extension,
  dylib = "-Wl,-rpath,@loader_path",
  so = "-Wl,-rpath,'$$ORIGIN'",
  dll = ""
)
makevars <- if (platform$extension == "dll") "Makevars.win" else "Makevars"
writeLines(
  sprintf("PKG_LIBS = -L$(CURDIR)/prebuilt -lquantion %s", rpath),
  file.path("src", makevars)
)
message("quantion: using the prebuilt ", asset, "; cargo is not needed")
