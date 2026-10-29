import { readFileSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const folder = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "public", "api");
const newVersion = process.argv[2]?.replace(/^v/, "");
const hasNewApi = process.argv.includes("--new-api");

if (!newVersion || !/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error("usage: npm run add-version -- 0.2.1 [--new-api]");
  process.exit(1);
}

const listPath = path.join(folder, "versions.json");
const list = JSON.parse(readFileSync(listPath, "utf8"));

if (list.versions.some((item) => item.version === newVersion)) {
  console.error(`${newVersion} is already listed`);
  process.exit(1);
}

const latestApi = list.versions.find((item) => item.version === list.latest).api;
let api = latestApi;

if (hasNewApi) {
  api = newVersion;
  const newPath = path.join(folder, `${api}.json`);
  if (existsSync(newPath)) {
    console.error(`${api}.json already exists`);
    process.exit(1);
  }
  writeFileSync(newPath, readFileSync(path.join(folder, `${latestApi}.json`)));
}

const versions = [{ version: newVersion, api }, ...list.versions];
writeFileSync(listPath, `${JSON.stringify({ latest: newVersion, versions }, null, 2)}\n`);

console.log(
  hasNewApi
    ? `added ${newVersion} with its own API file public/api/${api}.json; edit it to match the new API`
    : `added ${newVersion}, sharing public/api/${api}.json`,
);
