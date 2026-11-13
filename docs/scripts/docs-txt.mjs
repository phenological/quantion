import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { convertLimits } from '../src/ask/convert.js';
import { limits } from '../src/ask/models.js';
import { docsAddress, siteAddress } from '../src/site.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const input = process.argv.slice(2);
const versionAt = input.indexOf('--version');
const pinned = versionAt >= 0 ? input[versionAt + 1]?.replace(/^v/, '') : null;
const isAll = input.includes('--all');
const folder = input.find(
  (item, index) =>
    !item.startsWith('--') && (versionAt < 0 || index !== versionAt + 1),
);
if (!folder || isAll === Boolean(pinned)) {
  console.error(
    'usage: node scripts/docs-txt.mjs <folder> (--all | --version 0.2.2)',
  );
  process.exit(1);
}
const output = resolve(root, folder);

const readJson = (path) => JSON.parse(readFileSync(join(root, path), 'utf8'));
const versions = readJson('public/api/versions.json');
const ionicCli =
  'https://github.com/phenological/ionic/blob/main/crates/cli/src/README.MD';

const cell = (text) =>
  String(text ?? '')
    .replaceAll('|', '\\|')
    .replaceAll('\n', ' ');
const code = (text) => (text ? `\`${cell(text)}\`` : '');
const fence = (language, text) =>
  `\`\`\`${language === 'js' ? 'javascript' : language}\n${text}\n\`\`\``;

function helperSection() {
  const fenced = (text) => `\`\`\`\n${text}\n\`\`\``;
  return [
    '## Known code problems: ask the helper with `/code{ }`',
    '',
    `${siteAddress} has a helper (the Ask button, bottom right).`,
    '',
    `The helper knows common problems in Quant·ion code and how to fix them, such as too many peaks, no peaks or a wrong time window. This documentation does not cover them. If your code runs but the results look wrong, ask the helper. Write a short question, then the code inside \`/code{ }\`. One question per message, at most ${limits.code} characters. Example:`,
    '',
    fenced(
      [
        'find_peaks returns too many peaks /code{',
        'from quantion import calculate_eic, find_peaks, parse_ion',
        'file = parse_ion("sample.ion")',
        'eic = calculate_eic(file, 260.103, {"from": 2, "to": 3})',
        'peaks = find_peaks(eic["x"], eic["y"], {"min_snr": 3})',
        '}',
      ].join('\n'),
    ),
  ].join('\n');
}

function convertSection() {
  const download = (file) =>
    `https://github.com/phenological/ionic/releases/latest/download/${file}`;
  return [
    '## Convert mzML and Ionic (.ion) files',
    '',
    'Pick the way that suits you best:',
    '',
    `- To test one file quickly: the converter on the website. Type \`/convert\` in the helper at ${siteAddress}. It runs in the browser, with nothing to install. Up to ${convertLimits.mzml / 1000} GB for mzML and ${convertLimits.ion} MB for .ion.`,
    '- To convert files often: the Quant·ion library. Open the file with parse_mzml or parse_ion, then save it with write_ion or write_mzml.',
    '- To convert from the command line, or from an AI assistant: the `ionic` command-line tool. It is one file to download and run, with nothing to install.',
    '',
    '### The `ionic` command-line tool',
    '',
    'Download the file for your platform and make it executable. These links always give the latest version:',
    '',
    `- macOS arm64: ${download('ionic-aarch64-apple-darwin')}`,
    `- macOS x86_64: ${download('ionic-x86_64-apple-darwin')}`,
    `- Linux x86_64: ${download('ionic-x86_64-unknown-linux-gnu')}`,
    `- Linux arm64: ${download('ionic-aarch64-unknown-linux-gnu')}`,
    `- Windows x86_64: ${download('ionic-x86_64-pc-windows-msvc.exe')}`,
    '',
    '```bash',
    `curl -fL "${download('ionic-aarch64-apple-darwin')}" -o ionic && chmod +x ionic`,
    './ionic convert -i data.mzML -o out/                # mzML to .ion',
    './ionic convert --ion-to-mzml -i data.ion -o out/   # .ion to mzML',
    '```',
    '',
    `A folder as input is converted with all its subfolders. All options: ${ionicCli}`,
  ].join('\n');
}

function documentFor(release) {
  const api = readJson(`public/api/${release.api}.json`);
  const languageLabels = Object.fromEntries(
    api.languages.map((l) => [l.key, l.label]),
  );
  const languageKeys = api.languages.map((l) => l.key);

  function rowTable(rows) {
    if (!rows?.length) return '';
    const head = ['Meaning', ...languageKeys.map((k) => languageLabels[k])];
    const lines = [
      `| ${head.join(' | ')} |`,
      `| ${head.map(() => '---').join(' | ')} |`,
    ];
    for (const row of rows) {
      const cells = languageKeys.map((key) => {
        const item = row.languages[key];
        if (!item) return '';
        let text = `${code(item.name)}: ${code(item.type)}`;
        if (item.default && item.default !== '—')
          text += ` (default ${code(item.default)})`;
        return text;
      });
      lines.push(`| ${[cell(row.meaning), ...cells].join(' | ')} |`);
    }
    return lines.join('\n');
  }

  function perLanguage(entry, field) {
    return languageKeys
      .filter((key) => entry.languages?.[key]?.[field])
      .map(
        (key) =>
          `${languageLabels[key]}:\n\n${fence(key, entry.languages[key][field])}`,
      )
      .join('\n\n');
  }

  function entryPage(entry) {
    const kind = entry.kind === 'function' ? 'Function' : 'Type';
    const parts = [
      `## ${entry.name}`,
      '',
      `${kind} in the group ${entry.group}.`,
    ];
    if (entry.experimental)
      parts.push('', `Warning: ${entry.warning ?? 'experimental feature.'}`);
    parts.push('', entry.summary ?? '');
    if (entry.expect) parts.push('', `What to expect: ${entry.expect}`);
    if (entry.languages)
      parts.push('', '### Signature', '', perLanguage(entry, 'signature'));
    if (entry.arguments?.length)
      parts.push('', '### Arguments', '', rowTable(entry.arguments));
    if (entry.returns) {
      parts.push('', '### Returns', '', entry.returns.meaning ?? '', '');
      for (const key of languageKeys)
        if (entry.returns.languages?.[key])
          parts.push(
            `- ${languageLabels[key]}: ${code(entry.returns.languages[key])}`,
          );
    }
    if (entry.fields?.length) {
      const title = entry.group === 'Enums' ? 'Values' : 'Fields';
      parts.push('', `### ${title}`, '', rowTable(entry.fields));
    }
    const example = perLanguage(entry, 'example');
    if (example) parts.push('', '### Example', '', example);
    return parts.join('\n').replace(/\n{3,}/g, '\n\n');
  }

  const install = languageKeys
    .filter((key) => api.overview.install?.[key])
    .map((key) => `- ${languageLabels[key]}: \`${api.overview.install[key]}\``)
    .join('\n');
  const parts = [
    [
      `# ${api.overview.title}`,
      '',
      `> ${api.overview.summary}`,
      '',
      `Documentation for version ${release.version} (API ${release.api}). The same functions, arguments and options exist in ${api.languages.map((l) => l.label).join(', ')}. Names are snake_case in Rust, Python and R, and camelCase in JavaScript. Only the functions, options and values listed here exist.`,
      '',
      `Every version has its own file at ${docsAddress('<version>')}. If you use another version, read that file instead.`,
      '',
      '## Install',
      '',
      install,
      '',
      convertSection(),
      '',
      helperSection(),
    ].join('\n'),
  ];
  for (const group of api.groups) {
    const entries = api.entries.filter((e) => e.group === group.name);
    if (!entries.length) continue;
    parts.push(`# ${group.name}\n\n${group.description}`);
    for (const entry of entries) parts.push(entryPage(entry));
  }
  return parts.join('\n\n') + '\n';
}

mkdirSync(output, { recursive: true });
if (isAll) {
  for (const release of versions.versions)
    writeFileSync(
      join(output, `v${release.version}.txt`),
      documentFor(release),
    );
  console.log(`docs: ${versions.versions.length} versions in ${output}`);
} else {
  const release = versions.versions.find((v) => v.version === pinned);
  if (!release) {
    console.error(`docs: version ${pinned} is not in public/api/versions.json`);
    process.exit(1);
  }
  writeFileSync(join(output, 'docs.txt'), documentFor(release));
  console.log(`docs: docs.txt for ${release.version} in ${output}`);
}
