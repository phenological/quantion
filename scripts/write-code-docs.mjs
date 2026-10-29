import { readFileSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const site = "https://phenological.github.io/quantion/";
const apiFolder = path.join(root, "docs", "public", "api");
const versionList = JSON.parse(readFileSync(path.join(apiFolder, "versions.json"), "utf8"));
const latestApi = versionList.versions.find((item) => item.version === versionList.latest).api;
const documentation = JSON.parse(readFileSync(path.join(apiFolder, `${latestApi}.json`), "utf8"));
const skippedArguments = new Set(["self", "warp", "this"]);
const missing = [];

function toCamelCase(name) {
  return name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

function toSnakeCase(name) {
  return name.replace(/([a-z])([A-Z])/g, "$1_$2").toLowerCase();
}

function getDisplayName(entry, language) {
  return entry.kind === "function" && language === "js" ? toCamelCase(entry.name) : entry.name;
}

function getLink(entry, language) {
  return `${site}?language=${language}#${encodeURIComponent(entry.name)}`;
}

function getArguments(entry, language) {
  return (entry.arguments ?? [])
    .map((row) => ({ cell: row.languages[language], meaning: row.meaning }))
    .filter(({ cell }) => cell && !skippedArguments.has(cell.name));
}

function getReturns(entry, language) {
  return entry.returns?.languages[language] ? entry.returns.meaning : null;
}

function readFile(relativePath) {
  return readFileSync(path.join(root, relativePath), "utf8").split("\n");
}

function saveFile(relativePath, lines) {
  writeFileSync(path.join(root, relativePath), lines.join("\n"));
}

function getIndent(line) {
  return line.slice(0, line.length - line.trimStart().length);
}

function getRustBlock(entry) {
  const lines = [`**${entry.name}** · ${entry.group}`, "", entry.summary];
  const argumentsList = getArguments(entry, "rust");
  if (argumentsList.length) {
    lines.push("");
    argumentsList.forEach(({ cell, meaning }) => lines.push(`- \`${cell.name}\`: ${meaning}`));
  }
  const returns = getReturns(entry, "rust");
  if (returns) lines.push("", `Returns: ${returns}`);
  lines.push("", `[Documentation](${getLink(entry, "rust")})`);
  return lines;
}

function writeRust(entry, relativePaths, pattern) {
  for (const relativePath of relativePaths) {
    const lines = readFile(relativePath);
    const index = lines.findIndex((line) => pattern.test(line));
    if (index < 0) continue;
    let start = index;
    while (start > 0 && /^\s*(#\[|\/\/\/)/.test(lines[start - 1])) start -= 1;
    const kept = lines.slice(start, index).filter((line) => !/^\s*\/\/\//.test(line));
    const indent = getIndent(lines[index]);
    const block = getRustBlock(entry).map((line) => `${indent}///${line ? ` ${line}` : ""}`);
    lines.splice(start, index - start, ...block, ...kept);
    saveFile(relativePath, lines);
    return true;
  }
  return false;
}

function getPythonBlock(entry) {
  const lines = [`${entry.name} · ${entry.group}`, "", entry.summary];
  const argumentsList = getArguments(entry, "python");
  if (argumentsList.length) {
    lines.push("", "Args:");
    argumentsList.forEach(({ cell, meaning }) => lines.push(`    ${cell.name}: ${meaning}`));
  }
  const returns = getReturns(entry, "python");
  if (returns) lines.push("", "Returns:", `    ${returns}`);
  lines.push("", `Documentation: ${getLink(entry, "python")}`);
  return lines;
}

function removePythonDocstring(lines, index) {
  if (!/^\s*"""/.test(lines[index] ?? "")) return;
  const single = lines[index].trim().length > 3 && lines[index].trim().endsWith('"""');
  let end = index;
  if (!single) {
    end += 1;
    while (end < lines.length && !lines[end].includes('"""')) end += 1;
  }
  lines.splice(index, end - index + 1);
}

function writePythonBody(entry, relativePath, pattern) {
  const lines = readFile(relativePath);
  const index = lines.findIndex((line) => pattern.test(line));
  if (index < 0) return false;
  let end = index;
  while (!/:\s*$/.test(lines[end])) end += 1;
  const indent = `${getIndent(lines[index])}    `;
  removePythonDocstring(lines, end + 1);
  const block = getPythonBlock(entry);
  const docstring = [`${indent}"""${block[0]}`, ...block.slice(1).map((line) => (line ? `${indent}${line}` : "")), `${indent}"""`];
  lines.splice(end + 1, 0, ...docstring);
  saveFile(relativePath, lines);
  return true;
}

function writePythonAssignedDoc(entry, relativePath, name) {
  const lines = readFile(relativePath);
  const index = lines.findIndex((line) => line.startsWith(`${name} = TypedDict(`));
  if (index < 0) return false;
  let end = index;
  while (!/\)\s*$/.test(lines[end])) end += 1;
  if ((lines[end + 1] ?? "").startsWith(`${name}.__doc__ = `)) {
    let docEnd = end + 1;
    while (!lines[docEnd].trimEnd().endsWith('"""') || docEnd === end + 1 && lines[docEnd].trim() === `${name}.__doc__ = """`) docEnd += 1;
    lines.splice(end + 1, docEnd - end);
  }
  const block = getPythonBlock(entry);
  lines.splice(end + 1, 0, `${name}.__doc__ = """${block[0]}`, ...block.slice(1), '"""');
  saveFile(relativePath, lines);
  return true;
}

function getJsBlock(entry, indent) {
  const lines = [`${getDisplayName(entry, "js")} · ${entry.group}`, "", entry.summary];
  const argumentsList = getArguments(entry, "js");
  if (argumentsList.length) {
    lines.push("");
    argumentsList.forEach(({ cell, meaning }) => lines.push(`@param ${cell.name} - ${meaning}`));
  }
  const returns = getReturns(entry, "js");
  if (returns) lines.push(`@returns ${returns}`);
  lines.push(`@see {@link ${getLink(entry, "js")} | Documentation}`);
  return [`${indent}/**`, ...lines.map((line) => `${indent} *${line ? ` ${line}` : ""}`), `${indent} */`];
}

function writeJs(entry, relativePaths, pattern) {
  for (const relativePath of relativePaths) {
    const lines = readFile(relativePath);
    const index = lines.findIndex((line) => pattern.test(line));
    if (index < 0) continue;
    let start = index;
    if (/^\s*\*\/\s*$/.test(lines[index - 1] ?? "")) {
      start = index - 1;
      while (start > 0 && !/^\s*\/\*\*/.test(lines[start])) start -= 1;
    } else if (/^\s*\/\*\*.*\*\/\s*$/.test(lines[index - 1] ?? "")) {
      start = index - 1;
    }
    lines.splice(start, index - start, ...getJsBlock(entry, getIndent(lines[index])));
    saveFile(relativePath, lines);
    return true;
  }
  return false;
}

function writeR(entry, relativePath) {
  if (!existsSync(path.join(root, relativePath))) return false;
  let text = readFileSync(path.join(root, relativePath), "utf8");
  const section = `\\section{Documentation}{\n\\url{${getLink(entry, "r")}}\n}\n`;
  text = text.replace(/\\section\{Documentation\}\{\n[^\n]*\n\}\n/, "");
  text = text.includes("\\examples{") ? text.replace("\\examples{", `${section}\\examples{`) : `${text.trimEnd()}\n${section}`;
  writeFileSync(path.join(root, relativePath), text);
  return true;
}

const rustFunctionFiles = ["core/src/files.rs", "core/src/traces.rs", "core/src/targets.rs", "core/src/features.rs", "core/src/models.rs"];
const rustTypeFiles = [
  "core/src/options.rs",
  "core/src/error.rs",
  "core/src/sample_file.rs",
  "core/src/files.rs",
  "core/src/utilities/structs.rs",
  "core/src/utilities/find_features.rs",
  "core/src/utilities/find_noise_level.rs",
  "core/src/utilities/fit_peak.rs",
  "core/src/utilities/ion_image.rs",
  "core/src/utilities/warp_model.rs",
  "core/src/utilities/calculate_eic.rs",
];
const pythonMethodFiles = { predict_rts: ["wrappers/python/quantion/warp_model.py"], calibrate: ["wrappers/python/quantion/warp_model.py"], dispose: ["wrappers/python/quantion/file.py", "wrappers/python/quantion/model.py", "wrappers/python/quantion/warp_model.py"] };
const jsMethodFiles = { predict_rts: ["wrappers/js/src/utilities/warpModel.ts"], calibrate: ["wrappers/js/src/utilities/warpModel.ts"], dispose: ["wrappers/js/src/utilities/sampleFile.ts", "wrappers/js/src/utilities/model.ts", "wrappers/js/src/utilities/warpModel.ts"] };
const jsTypeFiles = ["wrappers/js/src/types/types.ts", "wrappers/js/src/utilities/sampleFile.ts", "wrappers/js/src/utilities/errors.ts"];
const pythonTypeFiles = ["wrappers/python/quantion/_types.py", "wrappers/python/quantion/_errors.py", "wrappers/python/quantion/file.py"];

for (const entry of documentation.entries) {
  const isFunction = entry.kind === "function";
  if (entry.languages.rust) {
    const name = entry.name;
    const written = isFunction
      ? writeRust(entry, rustFunctionFiles, new RegExp(`^\\s*pub fn ${name}\\(`))
      : writeRust(entry, rustTypeFiles, new RegExp(`^\\s*pub (struct|enum) ${name}\\b`));
    if (!written) missing.push(`rust ${name}`);
  }
  if (entry.languages.python) {
    let written = false;
    if (pythonMethodFiles[entry.name]) {
      written = pythonMethodFiles[entry.name].map((file) => writePythonBody(entry, file, new RegExp(`^\\s+def ${entry.name}\\(`))).some(Boolean);
    } else if (isFunction) {
      written = writePythonBody(entry, "wrappers/python/quantion/api.py", new RegExp(`^def ${entry.name}\\(`));
    } else {
      written = pythonTypeFiles.some((file) => writePythonBody(entry, file, new RegExp(`^class ${entry.name}\\b`)) || writePythonAssignedDoc(entry, file, entry.name));
    }
    if (!written) missing.push(`python ${entry.name}`);
  }
  if (entry.languages.js) {
    const name = getDisplayName(entry, "js");
    let written = false;
    if (jsMethodFiles[entry.name]) {
      written = jsMethodFiles[entry.name].map((file) => writeJs(entry, [file], new RegExp(`^\\s+(async )?${name}\\(`))).some(Boolean);
    } else if (isFunction) {
      written = writeJs(entry, ["wrappers/js/src/utilities/api.ts"], new RegExp(`^export (async )?function ${name}\\(`));
    } else {
      written = writeJs(entry, jsTypeFiles, new RegExp(`^export (interface|const|class|type) ${name}\\b`));
    }
    if (!written) missing.push(`js ${name}`);
  }
  if (entry.languages.r) {
    const rdName = isFunction ? entry.name : entry.name === "Header" ? "header" : toSnakeCase(entry.name);
    if (!writeR(entry, `wrappers/r/man/${rdName}.Rd`) && (isFunction || entry.name.endsWith("Options") || entry.name === "Header")) {
      missing.push(`r ${rdName}`);
    }
  }
}

console.log(missing.length ? `not found:\n  ${missing.join("\n  ")}` : "every item has its documentation");
