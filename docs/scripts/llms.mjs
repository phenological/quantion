// Writes the documentation for LLMs into dist/ after `vite build`:
//   llms.txt          index with one link per page (https://llmstxt.org)
//   llms-full.txt     every page in one file
//   llms/<name>.md    one page per function and type
// Everything comes from public/api/<latest>.json.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { convertLimits } from '../src/ask/convert.js';
import { limits } from '../src/ask/models.js';
import { siteAddress } from '../src/site.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const output = resolve(root, process.argv[2] ?? 'dist');

const readJson = (path) => JSON.parse(readFileSync(join(root, path), 'utf8'));
const versions = readJson('public/api/versions.json');
const latest = versions.versions.find((v) => v.version === versions.latest);
const api = readJson(`public/api/${latest.api}.json`);

const languageLabels = Object.fromEntries(
  api.languages.map((l) => [l.key, l.label]),
);
const languageKeys = api.languages.map((l) => l.key);
const entriesByName = new Map(api.entries.map((e) => [e.name, e]));

const pageAddress = (name) => `${siteAddress}/llms/${name}.md`;
const humanAddress = (name) => `${siteAddress}/#${encodeURIComponent(name)}`;
const cell = (text) =>
  String(text ?? '')
    .replaceAll('|', '\\|')
    .replaceAll('\n', ' ');
const code = (text) => (text ? `\`${cell(text)}\`` : '');
const fence = (language, text) =>
  `\`\`\`${language === 'js' ? 'javascript' : language}\n${text}\n\`\`\``;

// One table row per argument or field, one column per language.
function rowTable(rows, withDefault) {
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
      if (withDefault && item.default && item.default !== '—')
        text += ` (default ${code(item.default)})`;
      return text;
    });
    lines.push(`| ${[cell(row.meaning), ...cells].join(' | ')} |`);
  }
  return lines.join('\n');
}

function signatures(entry) {
  return languageKeys
    .filter((key) => entry.languages?.[key]?.signature)
    .map(
      (key) =>
        `${languageLabels[key]}:\n\n${fence(key, entry.languages[key].signature)}`,
    )
    .join('\n\n');
}

function examples(entry) {
  return languageKeys
    .filter((key) => entry.languages?.[key]?.example)
    .map(
      (key) =>
        `${languageLabels[key]}:\n\n${fence(key, entry.languages[key].example)}`,
    )
    .join('\n\n');
}

// Types named in the arguments that have their own entry (options, inputs).
function linkedTypes(entry) {
  const names = new Set();
  for (const argument of entry.arguments ?? []) {
    for (const item of Object.values(argument.languages)) {
      for (const word of String(item?.type ?? '').match(/[A-Za-z_]\w*/g) ??
        []) {
        const found = entriesByName.get(word);
        if (found && found.kind === 'type') names.add(word);
      }
    }
  }
  return [...names].map((name) => entriesByName.get(name));
}

function entryPage(entry, { inline = true } = {}) {
  const parts = [`# ${entry.name}`, ''];
  const kind = entry.kind === 'function' ? 'Function' : 'Type';
  parts.push(
    `${kind} in the group ${entry.group}. Human page ${humanAddress(entry.name)}`,
  );
  if (entry.experimental)
    parts.push('', `Warning: ${entry.warning ?? 'experimental feature.'}`);
  parts.push('', entry.summary ?? '');
  if (entry.expect) parts.push('', `What to expect: ${entry.expect}`);

  if (entry.languages) parts.push('', '## Signature', '', signatures(entry));
  if (entry.arguments?.length)
    parts.push('', '## Arguments', '', rowTable(entry.arguments, true));
  if (entry.returns) {
    parts.push('', '## Returns', '', entry.returns.meaning ?? '', '');
    for (const key of languageKeys)
      if (entry.returns.languages?.[key])
        parts.push(
          `- ${languageLabels[key]}: ${code(entry.returns.languages[key])}`,
        );
  }
  if (entry.fields?.length) {
    const title = entry.group === 'Enums' ? 'Values' : 'Fields';
    parts.push('', `## ${title}`, '', rowTable(entry.fields, true));
  }
  if (entry.languages && examples(entry))
    parts.push('', '## Example', '', examples(entry));

  if (inline && entry.kind === 'function') {
    for (const type of linkedTypes(entry)) {
      parts.push(
        '',
        `## ${type.name}`,
        '',
        type.summary ?? '',
        '',
        rowTable(type.fields, true),
      );
    }
  }
  return parts.join('\n').replace(/\n{3,}/g, '\n\n') + '\n';
}

// Tells AI assistants what the helper on the website adds to these docs:
// known problems in code with their fixes, and the file converter.
function websiteSection() {
  const fenced = (text) => `\`\`\`\n${text}\n\`\`\``;
  return [
    '## The helper on the website',
    '',
    `${siteAddress} has a helper (the Ask button, bottom right). It adds two things to this documentation.`,
    '',
    '### Known code problems with `/code{ }`',
    '',
    `The helper knows common problems in Quant·ion code and how to fix them, such as too many peaks, no peaks or a wrong time window. This documentation does not. If the user's code runs but the results look wrong, write a message for them to paste in the helper. Use a short question, then the code inside \`/code{ }\`. One question per message, at most ${limits.code} characters. Example:`,
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
    '',
    '### Convert files with `/convert`',
    '',
    `Type \`/convert\` in the helper to convert mzML to Ionic (.ion), or .ion to mzML. Nothing to install, and the file stays on the user's computer. Up to ${convertLimits.mzml / 1000} GB for mzML and ${convertLimits.ion} MB for .ion. For bigger files, or in code, use write_ion and write_mzml.`,
  ].join('\n');
}

function header(lead = []) {
  const { overview } = api;
  const install = languageKeys
    .filter((key) => overview.install?.[key])
    .map((key) => `- ${languageLabels[key]}: \`${overview.install[key]}\``)
    .join('\n');
  return [
    `# ${overview.title}`,
    '',
    `> ${overview.summary}`,
    '',
    ...lead,
    `Documentation for version ${latest.version} (API ${latest.api}). The same functions, arguments and options exist in ${api.languages.map((l) => l.label).join(', ')}. Names are snake_case in Rust, Python and R, and camelCase in JavaScript. Use only the functions, options and values listed here.`,
    '',
    websiteSection(),
    '',
    '## Install',
    '',
    install,
  ].join('\n');
}

function indexFile() {
  const lines = [
    header([
      `Full documentation in one file: ${siteAddress}/llms-full.txt. Read it to get every function, argument, option, field and default in one fetch. The links below are the same pages one by one.`,
      '',
    ]),
    '',
  ];
  for (const group of api.groups) {
    const entries = api.entries.filter((e) => e.group === group.name);
    if (!entries.length) continue;
    lines.push(`## ${group.name}`, '', group.description, '');
    for (const entry of entries)
      lines.push(
        `- [${entry.name}](${pageAddress(entry.name)}): ${cell(entry.summary)}`,
      );
    lines.push('');
  }
  lines.push(
    '## Optional',
    '',
    `- [Full documentation in one file](${siteAddress}/llms-full.txt): every page above in one file`,
    `- [API as JSON](${siteAddress}/api/${latest.api}.json): the data this documentation is made from`,
    `- [Source code](https://github.com/phenological/quantion)`,
    `- [Report an issue](https://github.com/phenological/quantion/issues)`,
  );
  return lines.join('\n') + '\n';
}

function fullFile() {
  const parts = [header()];
  for (const group of api.groups) {
    const entries = api.entries.filter((e) => e.group === group.name);
    if (!entries.length) continue;
    parts.push(`# Group ${group.name}\n\n${group.description}`);
    for (const entry of entries)
      parts.push(
        entryPage(entry, { inline: false }).replace(/^(#+) /gm, '#$1 '),
      );
  }
  return parts.join('\n\n') + '\n';
}

mkdirSync(join(output, 'llms'), { recursive: true });
writeFileSync(join(output, 'llms.txt'), indexFile());
writeFileSync(join(output, 'llms-full.txt'), fullFile());
for (const entry of api.entries)
  writeFileSync(join(output, 'llms', `${entry.name}.md`), entryPage(entry));

console.log(
  `llms: ${api.entries.length} pages, llms.txt and llms-full.txt in ${output}`,
);
