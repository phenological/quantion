import { levenshteinDistance } from 'str-utilities';

import { getDisplayName } from '../names.js';
import {
  builtinNames,
  checkBrackets,
  detectLanguage,
  findClose,
  findDefinedNames,
  findKeywordTypos,
  findLooseKeywordTypos,
  findSyntaxHints,
  maskCode,
  memberNames,
  readTopLevel,
} from './code.js';
import {
  findCall,
  findDefinition,
  getKind,
  getOptionsType,
  getPlainName,
  toSnakeCase,
} from './snippet.js';

const callPattern = /\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/g;
const languages = ['python', 'js', 'r', 'rust'];
export const languageLabels = {
  python: 'Python',
  js: 'JavaScript',
  r: 'R',
  rust: 'Rust',
};

function getAllowedDistance(name) {
  return name.length >= 10 ? 3 : name.length >= 6 ? 2 : 1;
}

function findClosest(name, candidates) {
  let best = null;
  for (const candidate of candidates) {
    const distance = levenshteinDistance(
      name.toLowerCase(),
      candidate.toLowerCase(),
    );
    if (!best || distance < best.distance) best = { name: candidate, distance };
  }
  if (best && best.distance <= getAllowedDistance(name)) return best.name;
  const letters = [...name.toLowerCase()].sort().join('');
  return (
    candidates.find(
      (candidate) =>
        name.length >= 3 &&
        [...candidate.toLowerCase()].sort().join('') === letters,
    ) ?? null
  );
}

function getFunctionNames(document, language) {
  const names = new Map();
  for (const entry of document.entries) {
    if (entry.kind === 'function')
      names.set(getDisplayName(entry, language), entry);
  }
  return names;
}

function getHelperNames(document) {
  const names = new Set([
    'function',
    'nrow',
    'ncol',
    'length',
    'paste',
    'paste0',
    'seq',
    'rep',
    'sapply',
    'lapply',
    'unlist',
    'is.na',
    'which',
    'names',
    'summary',
    'round',
    'mean',
    'sum',
    'max',
    'min',
    'abs',
    'sqrt',
    'exp',
    'log',
    'open',
    'enumerate',
    'zip',
    'sorted',
    'isinstance',
    'int',
    'float',
    'str',
    'dict',
    'tuple',
    'set',
    'Number',
    'String',
    'Array',
    'Object',
    'Boolean',
    'parseInt',
    'parseFloat',
    'fetch',
    'require',
    'Promise',
    'Error',
    'if',
    'for',
    'while',
    'switch',
    'return',
    'repeat',
    'match',
    'catch',
    'new',
    'typeof',
    'print',
    'len',
    'range',
    'library',
    'head',
    'data',
    'frame',
    'list',
    'vec',
    'Some',
    'Ok',
    'drop',
    'main',
    'log',
    'c',
    'cat',
    'sprintf',
  ]);
  for (const entry of document.entries) {
    if (entry.kind !== 'function') names.add(entry.name);
    if (entry.group === 'Options') names.add(toSnakeCase(entry.name));
  }
  return names;
}

function getOptionFields(entry, document) {
  const fields = [];
  for (const field of getOptionsType(entry, document)?.fields ?? []) {
    fields.push({ field, owner: entry });
    const kind = getKind(
      (field.languages.rust?.type ?? '').replace(/^Vec<(.+)>$/, '$1'),
      document,
    );
    if (kind.kind !== 'options') continue;
    for (const inner of kind.entry.fields ?? [])
      fields.push({ field: inner, owner: entry });
  }
  return fields;
}

function findOptionElsewhere(name, document, language, entry) {
  for (const candidate of document.entries) {
    if (candidate.kind !== 'function' || candidate === entry) continue;
    for (const { field } of getOptionFields(candidate, document)) {
      if (field.languages[language]?.name === name) return candidate;
    }
  }
  return null;
}

function getOptionsText(code, call, language, entry) {
  const last = call.pieces[call.pieces.length - 1];
  if (!last) return null;
  const slots = [
    ...new Set(
      (entry.arguments ?? [])
        .map((item) => item.languages[language]?.name?.split('.')[0])
        .filter(Boolean),
    ),
  ];
  const optionsSlot = slots.indexOf('options');
  const raw = code.slice(last.start, last.end).trim();
  if (!/^options\s*=/.test(raw) && call.pieces.length <= optionsSlot)
    return null;
  let text = raw.replace(/^options\s*=\s*/, '');
  if (/^[A-Za-z_]\w*$/.test(text)) {
    const definition = findDefinition(code, text);
    if (!definition) return null;
    text = code.slice(definition.start, definition.end);
  }
  if (language === 'python' && !text.startsWith('{')) return null;
  if (language === 'js' && !text.startsWith('{')) return null;
  if (language === 'r' && !/_options\(/.test(text)) return null;
  if (language === 'rust' && !/Options\s*\{/.test(text)) return null;
  return text;
}

function findKeys(text, language) {
  const keys = [];
  if (language === 'python') {
    for (const match of text.matchAll(/["']([\w-]+)["']\s*:/g))
      keys.push({ name: match[1], index: match.index });
    return keys;
  }
  const bare = text.replace(/"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'/g, (literal) =>
    ' '.repeat(literal.length),
  );
  const pattern =
    language === 'r'
      ? /([A-Za-z_][\w.]*)\s*=(?!=)/g
      : /([A-Za-z_]\w*)\s*:(?!:)/g;
  for (const match of bare.matchAll(pattern))
    keys.push({ name: match[1], index: match.index });
  return keys;
}

function readValueText(text, key) {
  const after = text.slice(
    key.index +
      key.name.length +
      (text[key.index] === '"' || text[key.index] === "'" ? 2 : 0),
  );
  const match = /^\s*["']?\s*[:=]\s*(["'][^"']*["']|(?:[\w.+-]|::)+)/.exec(
    after,
  );
  return match ? match[1] : null;
}

const enumWords = {
  MINUTES: ['min', 'mins', 'minute', 'minutes', 'm'],
  SECONDS: ['s', 'sec', 'secs', 'second', 'seconds'],
  GAUSSIAN: ['gauss', 'gaussian', 'normal'],
  EMG: [
    'emg',
    'exponential',
    'exp_gaussian',
    'exponentially_modified_gaussian',
  ],
};

function checkEnumValue(type, valueText, language) {
  const values = (type.fields ?? []).map((field) =>
    (field.languages.python?.name ?? '').replace(/"/g, ''),
  );
  const quote = /^["']/.exec(valueText)?.[0] ?? '';
  const prefix =
    new RegExp(`^${type.name}(\\.|::)`).exec(
      valueText.slice(quote.length),
    )?.[0] ?? '';
  const raw = valueText
    .slice(quote.length + prefix.length)
    .replace(/["']$/, '');
  if (values.includes(raw) && (quote || prefix || language === 'rust'))
    return null;
  const right =
    values.find((value) => value.toLowerCase() === raw.toLowerCase()) ??
    values.find((value) => enumWords[value]?.includes(raw.toLowerCase())) ??
    findClosest(raw.toUpperCase(), values);
  if (!right) return null;
  const separator = language === 'rust' ? '::' : '.';
  let to;
  if (prefix) to = `${type.name}${separator}${right}`;
  else if (quote) to = `${quote}${right}${quote}`;
  else if (language === 'r') to = `"${right}"`;
  else to = `${type.name}${separator}${right}`;
  if (to === valueText) return null;
  return {
    from: valueText,
    to,
    reason: `\`${type.name}\` values are ${values.map((value) => `\`${value}\``).join(' and ')}.`,
  };
}

function checkValue(field, valueText, document, language) {
  if (!valueText) return null;
  const kind = getKind(field.languages.rust?.type ?? '', document);
  const quoted = /^["'](.*)["']$/.exec(valueText);
  if (
    (kind.kind === 'number' || kind.kind === 'integer') &&
    quoted &&
    !Number.isNaN(Number(quoted[1]))
  ) {
    const number =
      language === 'rust' && kind.kind === 'number' && !quoted[1].includes('.')
        ? `${quoted[1]}.0`
        : quoted[1];
    return {
      from: valueText,
      to: number,
      reason: `\`${field.languages[language].name}\` takes a number, not text.`,
    };
  }
  const nulls = { python: 'None', rust: 'None', r: 'NULL' };
  if (
    nulls[language] &&
    ['null', 'None', 'NULL', 'undefined', 'nil'].includes(valueText) &&
    valueText !== nulls[language]
  )
    return {
      from: valueText,
      to: nulls[language],
      reason: `${languageLabels[language]} writes an empty value as ${nulls[language]}.`,
    };
  if (
    language === 'rust' &&
    kind.kind === 'number' &&
    /^-?\d+$/.test(valueText)
  ) {
    return {
      from: valueText,
      to: `${valueText}.0`,
      reason: `In Rust, \`${field.languages.rust.name}\` is a decimal number (f64), so write ${valueText}.0.`,
    };
  }
  if (
    language === 'rust' &&
    kind.kind === 'integer' &&
    /^-?\d+\.0*$/.test(valueText)
  ) {
    return {
      from: valueText,
      to: valueText.split('.')[0],
      reason: `In Rust, \`${field.languages.rust.name}\` is a whole number, so write ${valueText.split('.')[0]}.`,
    };
  }
  if (kind.kind === 'enum')
    return checkEnumValue(kind.entry, valueText, language);
  if (kind.kind === 'boolean') {
    const right = {
      python: { true: 'True', false: 'False' },
      r: { true: 'TRUE', false: 'FALSE' },
    }[language] ?? { true: 'true', false: 'false' };
    const words = {
      yes: 'true',
      on: 'true',
      t: 'true',
      no: 'false',
      off: 'false',
      f: 'false',
    };
    const plain = valueText.replace(/^["'](.*)["']$/, '$1').toLowerCase();
    const lower = words[plain] ?? plain;
    if (right[lower] && right[lower] !== valueText)
      return {
        from: valueText,
        to: right[lower],
        reason: `${languageLabels[language]} writes booleans as ${right.true}/${right.false}.`,
      };
  }
  return null;
}

function getOptionOwners(entry, document) {
  const owners = [entry.name];
  for (const field of getOptionsType(entry, document)?.fields ?? []) {
    const kind = getKind(field.languages.rust?.type ?? '', document);
    if (kind.kind === 'options') owners.push(kind.entry.name);
  }
  const optionsType = getOptionsType(entry, document);
  if (optionsType) owners.push(optionsType.name);
  return owners;
}

function checkOptionKeys(text, body, fields, label, scope, state) {
  const { language, document, fixes, notes, guesses, unknown } = state;
  const valid = new Map(
    fields.map((field) => [field.languages[language]?.name, field]),
  );
  for (const key of findKeys(text, language)) {
    const field = valid.get(key.name);
    if (field) {
      const valueFix = checkValue(
        field,
        readValueText(body, key),
        document,
        language,
      );
      if (
        valueFix &&
        !fixes.some(
          (fix) => fix.from === valueFix.from && fix.context === key.name,
        )
      )
        fixes.push({ ...valueFix, context: key.name });
      continue;
    }
    if (
      scope.skipSeen &&
      (fixes.some((fix) => fix.from === key.name) ||
        notes.some((note) => note.startsWith(`\`${key.name}\``)))
    )
      continue;
    const otherForm = fields.find((item) =>
      languages.some((other) => item.languages[other]?.name === key.name),
    );
    if (otherForm) {
      const right = otherForm.languages[language].name;
      fixes.push({
        from: key.name,
        to: right,
        reason: `${languageLabels[language]} names this option \`${right}\`.`,
      });
      continue;
    }
    const elsewhere = scope.elsewhere?.(key.name);
    const guess = fields.find(
      (item) => getPlainName(item) === guesses[`${scope.guessKey}:${key.name}`],
    );
    const right =
      findClosest(key.name, [...valid.keys()].filter(Boolean)) ??
      (elsewhere ? null : guess?.languages[language]?.name);
    if (right) {
      fixes.push({
        from: key.name,
        to: right,
        reason: `\`${key.name}\` is not an option of ${label}; did you mean \`${right}\`?`,
      });
      continue;
    }
    if (elsewhere) {
      notes.push(
        `\`${key.name}\` is an option of \`${getDisplayName(elsewhere, language)}\`, not of ${label}.`,
      );
      continue;
    }
    unknown.push({
      kind: 'option',
      name: key.name,
      function: scope.guessKey,
      owners: scope.owners,
      fields: fields.map(getPlainName),
    });
    notes.push(
      `\`${key.name}\` is not an option of ${label}. Its options: ${[
        ...valid.keys(),
      ]
        .slice(0, 6)
        .map((name) => `\`${name}\``)
        .join(', ')}…`,
    );
  }
}

function reviewOptions(code, entry, state) {
  const { language, document } = state;
  const call = findCall(code, getDisplayName(entry, language));
  const text = call && getOptionsText(code, call, language, entry);
  if (!text) return;
  const fields = getOptionFields(entry, document).map(({ field }) => field);
  checkOptionKeys(
    text,
    text,
    fields,
    `\`${getDisplayName(entry, language)}\``,
    {
      guessKey: entry.name,
      owners: getOptionOwners(entry, document),
      elsewhere: (name) => findOptionElsewhere(name, document, language, entry),
    },
    state,
  );
}

function getRecordFields(entry, document) {
  const fields = getOptionFields(entry, document).map(({ field }) => field);
  for (const argument of entry.arguments ?? []) {
    const type = (argument.languages.rust?.type ?? '')
      .replace(/^&(mut )?/, '')
      .replace(/^\[|^Vec<|[\]>]$/g, '');
    const record = document.entries.find(
      (item) => item.name === type && item.group !== 'Options',
    );
    fields.push(...(record?.fields ?? []));
  }
  return fields;
}

function readArgumentValues(code, call, entry, language, document) {
  const values = [];
  const slots = [
    ...new Set(
      (entry.arguments ?? []).map(
        (item) => item.languages[language]?.name?.split('.')[0],
      ),
    ),
  ];
  for (const argument of entry.arguments ?? []) {
    const name = argument.languages[language]?.name;
    if (!name || name.includes('.')) continue;
    const piece = call.pieces[slots.indexOf(name)];
    const kind = getKind(argument.languages.rust?.type ?? '', document).kind;
    if (!piece || !['number', 'integer', 'text'].includes(kind)) continue;
    let text = code
      .slice(piece.start, piece.end)
      .trim()
      .replace(/^[A-Za-z_]\w*\s*=(?!=)\s*/, '');
    if (/^[A-Za-z_]\w*$/.test(text)) {
      const definition = findDefinition(code, text);
      text = definition ? code.slice(definition.start, definition.end) : '';
    }
    if (/^(-?\d+(\.\d+)?|"[^"]*"|'[^']*')$/.test(text.trim()))
      values.push({
        function: entry.name,
        name: getPlainName(argument),
        value: text.trim().replace(/^["']|["']$/g, ''),
      });
  }
  return values;
}

function readCodeValues(code, entry, language, document) {
  const call = findCall(code, getDisplayName(entry, language));
  if (!call) return [];
  const values = readArgumentValues(code, call, entry, language, document);
  const fields = new Map(
    getRecordFields(entry, document).map((field) => [
      field.languages[language]?.name,
      field,
    ]),
  );
  for (const key of findKeys(code, language)) {
    const field = fields.get(key.name);
    const value = field && readValueText(code, key);
    if (value && !values.some((item) => item.name === getPlainName(field)))
      values.push({
        function: entry.name,
        name: getPlainName(field),
        value: value.replace(/^["']|["']$/g, ''),
      });
  }
  return values;
}

function applyFixes(code, fixes) {
  let fixed = code;
  for (const fix of fixes) {
    if (fix.context)
      fixed = fixed.replace(
        new RegExp(
          `(${fix.context}["']?\\s*[:=]\\s*)${fix.from.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`,
        ),
        `$1${fix.to}`,
      );
    else if (fix.whole) fixed = fixed.replace(fix.from, fix.to);
    else if (/^[A-Za-z_]\w*$/.test(fix.from))
      fixed = fixed.replace(
        new RegExp(`(?<![\\w.])${fix.from}(?!\\w)`, 'g'),
        fix.to,
      );
  }
  return fixed;
}

function findImportedNames(code) {
  const names = [];
  const lists = [
    ...code.matchAll(/\buse\s+quantion::\{([^}]*)\}/g),
    ...code.matchAll(/\buse\s+quantion::(\w+)/g),
    ...code.matchAll(/\bfrom\s+quantion\s+import\s+(\([^)]*\)|[\w ,]+)/g),
    ...code.matchAll(/\bimport\s*\{([^}]*)\}\s*from\s*["']quantion["']/g),
  ];
  for (const match of lists) {
    for (const piece of match[1].replace(/[()]/g, '').split(',')) {
      const name = piece
        .trim()
        .split(/\s+as\s+/)[0]
        .trim();
      if (/^[A-Za-z_]\w*$/.test(name) && !names.includes(name))
        names.push(name);
    }
  }
  return names;
}

const definitionPattern = /\b(def|fn|function|class|struct|enum|impl)\s+$/;

const rustTypes = new Set([
  'Vec',
  'String',
  'Option',
  'Some',
  'None',
  'Result',
  'Ok',
  'Err',
  'Box',
  'Rc',
  'Arc',
  'Default',
  'HashMap',
  'HashSet',
  'BTreeMap',
  'Path',
  'PathBuf',
  'Self',
  'Duration',
  'Instant',
]);

function findMissingDefaults(code, document) {
  const fixes = [];
  const masked = maskCode(code, 'rust');
  for (const match of masked.matchAll(/\b([A-Z]\w*Options)\s*\{/g)) {
    const type = document.entries.find(
      (entry) => entry.name === match[1] && entry.group === 'Options',
    );
    const open = match.index + match[0].length - 1;
    const close = findClose(masked, open);
    if (!type || close < 0) continue;
    const topLevel = readTopLevel(code.slice(open + 1, close));
    if (topLevel.includes('..')) continue;
    const keys = [...topLevel.matchAll(/\b([a-z_]\w*)\s*:(?!:)/g)].length;
    if (keys >= (type.fields ?? []).length) continue;
    const literal = code.slice(match.index, close + 1);
    fixes.push({
      from: literal,
      to: literal.replace(/\s*,?\s*\}$/, ', ..Default::default() }'),
      whole: true,
      reason: `\`${match[1]}\` must list every field, or end with \`..Default::default()\` to use the defaults for the rest.`,
    });
  }
  return fixes;
}

function findMissingArguments(code, found, document) {
  const fixes = [];
  for (const entry of found) {
    const call = findCall(code, entry.name);
    if (!call) continue;
    const slots = [
      ...new Set(
        (entry.arguments ?? [])
          .map((item) => item.languages.rust?.name?.split('.')[0])
          .filter(Boolean),
      ),
    ];
    const type = getOptionsType(entry, document)?.name;
    if (
      call.pieces.length >= slots.length ||
      (type && code.slice(call.start, call.end).includes(type))
    )
      continue;
    const missing = (entry.arguments ?? [])
      .filter(
        (item) =>
          slots.indexOf(item.languages.rust?.name) >= call.pieces.length,
      )
      .map((item) =>
        type === item.languages.rust?.type
          ? `${item.languages.rust.type}::default()`
          : null,
      );
    const literal = code.slice(
      call.start - entry.name.length - 1,
      call.end + 1,
    );
    if (missing.length > 0 && missing.every(Boolean))
      fixes.push({
        from: literal,
        to: literal.replace(
          /\s*\)$/,
          `${call.pieces.length > 0 ? ', ' : ''}${missing.join(', ')})`,
        ),
        whole: true,
        reason: `In Rust, \`${entry.name}\` needs every argument. Pass \`${missing.join('`, `')}\` to use the default options.`,
      });
  }
  return fixes;
}

function getRConstructor(type) {
  return /^(\w+)\(/.exec(type.languages.r?.signature ?? '')?.[1] ?? null;
}

function findOptionLiterals(code, language, document) {
  const types = document.entries.filter((entry) => entry.group === 'Options');
  const literals = [];
  const masked = maskCode(code, language);
  if (language === 'rust') {
    for (const match of masked.matchAll(/\b([A-Z]\w*)\s*\{/g)) {
      const type = types.find((entry) => entry.name === match[1]);
      const open = match.index + match[0].length - 1;
      const close = findClose(masked, open);
      if (type && close > 0)
        literals.push({
          type,
          label: type.name,
          body: code.slice(open + 1, close),
        });
    }
  }
  if (language === 'r') {
    for (const match of masked.matchAll(/\b(\w+_options)\s*\(/g)) {
      const type = types.find((entry) => getRConstructor(entry) === match[1]);
      const open = match.index + match[0].length - 1;
      const close = findClose(masked, open);
      if (type && close > 0)
        literals.push({
          type,
          label: match[1],
          body: code.slice(open + 1, close),
        });
    }
  }
  return literals;
}

function reviewOptionLiterals(code, state) {
  for (const { type, label, body } of findOptionLiterals(
    code,
    state.language,
    state.document,
  ))
    checkOptionKeys(
      readTopLevel(body),
      body,
      type.fields ?? [],
      `\`${label}\``,
      {
        guessKey: type.name,
        owners: [type.name],
        skipSeen: true,
      },
      state,
    );
}

function findRConstructorFixes(code, document) {
  const fixes = [];
  const types = document.entries.filter((entry) => entry.group === 'Options');
  const constructors = types.map(getRConstructor).filter(Boolean);
  for (const match of code.matchAll(
    /\b([A-Za-z]\w*(?:Options|_options))\s*\(/g,
  )) {
    const name = match[1];
    if (constructors.includes(name) || fixes.some((fix) => fix.from === name))
      continue;
    const type = types.find((entry) => entry.name === name);
    const right = type
      ? getRConstructor(type)
      : findClosest(name, constructors);
    if (right)
      fixes.push({
        from: name,
        to: right,
        reason: `R builds these options with \`${right}()\`.`,
      });
  }
  return fixes;
}

function findForeignImports(code) {
  const names = new Set();
  const lists = [
    ...code.matchAll(
      /\bfrom\s+(?!quantion\b)[\w.]+\s+import\s+(\([^)]*\)|[\w ,]+)/g,
    ),
    ...code.matchAll(
      /\bimport\s*\{([^}]*)\}\s*from\s*["'](?!quantion["'])[^"']+["']/g,
    ),
    ...code.matchAll(/\buse\s+(?!quantion::)[\w:]+::\{([^}]*)\}/g),
  ];
  for (const match of lists)
    for (const piece of match[1].replace(/[()]/g, '').split(','))
      names.add(
        piece
          .trim()
          .split(/\s+as\s+/)
          .pop()
          .trim(),
      );
  for (const match of code.matchAll(/\buse\s+(?!quantion::)[\w:]+::(\w+)\s*;/g))
    names.add(match[1]);
  for (const match of code.matchAll(
    /^\s*import\s+([\w.]+)(?:\s+as\s+(\w+))?/gm,
  ))
    names.add(match[2] ?? match[1]);
  return names;
}

function findFixLine(fix, code, masked, named) {
  const escaped = fix.from.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  let index = -1;
  if (fix.context)
    index = code.search(
      new RegExp(`${fix.context}["']?\\s*[:=]\\s*${escaped}`),
    );
  else if (fix.whole) {
    index = code.indexOf(fix.from);
    if (index < 0) index = named.indexOf(fix.from);
  } else {
    index = masked.search(new RegExp(`(?<![\\w.])${escaped}(?!\\w)`));
    if (index < 0)
      index = code.search(new RegExp(`(?<![\\w.])${escaped}(?!\\w)`));
  }
  return index < 0 ? null : code.slice(0, index).split('\n').length;
}

function findMemberTypos(masked, language, defined) {
  const fixes = [];
  const add = (from, to, kind) => {
    if (to && from !== to && !fixes.some((fix) => fix.from === from))
      fixes.push({
        from,
        to,
        reason: `There is no ${kind} \`${from}\`; did you mean \`${to}\`?`,
      });
  };
  if (language === 'js')
    for (const match of masked.matchAll(
      /(?<![\w.])([A-Za-z_]\w*)\.([A-Za-z_]\w*)\s*\(/g,
    )) {
      const [, object, member] = match;
      if (defined.has(object)) continue;
      const known = memberNames[object]
        ? object
        : findClosest(
            object,
            Object.keys(memberNames).filter((name) => name !== 'rust'),
          );
      if (!known) continue;
      add(object, known, 'object');
      if (!memberNames[known].includes(member))
        add(
          member,
          findClosest(member, memberNames[known]),
          `\`${known}\` method`,
        );
    }
  if (language === 'rust') {
    for (const match of masked.matchAll(/\b([a-z_]\w*)!\s*[([]/g))
      if (!builtinNames.rust.includes(match[1]))
        add(match[1], findClosest(match[1], builtinNames.rust), 'macro');
    for (const match of masked.matchAll(/\.([a-z_]\w*)\s*\(/g)) {
      const name = match[1];
      if (memberNames.rust.includes(name)) continue;
      add(name, findClosest(name, memberNames.rust), 'method');
    }
  }
  return fixes;
}

export function reviewCode(code, document, fallbackLanguage, guesses = {}) {
  const language = detectLanguage(code, fallbackLanguage);
  const functionNames = getFunctionNames(document, language);
  const helperNames = getHelperNames(document);
  const fixes = [];
  const notes = checkBrackets(code, language);
  const found = [];
  const unknown = [];
  for (const typo of findKeywordTypos(maskCode(code, language)))
    if (!fixes.some((fix) => fix.from === typo.word))
      fixes.push({
        from: typo.word,
        to: typo.keyword,
        reason: `\`${typo.word}\` should be \`${typo.keyword}\`.`,
      });
  const typeNames = new Set(
    document.entries
      .filter((entry) => entry.kind !== 'function')
      .map((entry) => entry.name),
  );

  for (const name of findImportedNames(applyFixes(code, fixes))) {
    if (functionNames.has(name)) {
      if (!found.includes(functionNames.get(name)))
        found.push(functionNames.get(name));
      continue;
    }
    if (typeNames.has(name)) continue;
    const closest = findClosest(name, [...functionNames.keys(), ...typeNames]);
    const guessed = document.entries.find(
      (entry) =>
        entry.kind === 'function' && entry.name === guesses[`:${name}`],
    );
    const right =
      closest ?? (guessed ? getDisplayName(guessed, language) : null);
    if (right) {
      fixes.push({
        from: name,
        to: right,
        reason: `Quant·ion has no \`${name}\`; did you mean \`${right}\`?`,
      });
      if (functionNames.has(right) && !found.includes(functionNames.get(right)))
        found.push(functionNames.get(right));
    } else {
      notes.push(`\`${name}\` is not part of Quant·ion.`);
      unknown.push({ kind: 'function', name });
    }
  }

  const masked = maskCode(code, language);
  const foreign = findForeignImports(code);
  const defined = findDefinedNames(masked);
  for (const typo of findLooseKeywordTypos(
    masked,
    language,
    new Set([
      ...defined,
      ...functionNames.keys(),
      ...foreign,
      ...document.entries.flatMap((entry) =>
        (entry.fields ?? []).map((field) => field.languages[language]?.name),
      ),
    ]),
  ))
    if (!fixes.some((fix) => fix.from === typo.word))
      fixes.push({
        from: typo.word,
        to: typo.keyword,
        reason: `\`${typo.word}\` should be \`${typo.keyword}\`.`,
      });
  if (!notes.some((note) => /^Line \d+:/.test(note)))
    notes.push(...findSyntaxHints(masked, language));
  fixes.push(...findMemberTypos(masked, language, defined));
  for (const match of masked.matchAll(callPattern)) {
    const name = match[1];
    if (foreign.has(name) || defined.has(name)) continue;
    if (definitionPattern.test(masked.slice(0, match.index))) continue;
    if (functionNames.has(name)) {
      const entry = functionNames.get(name);
      if (!found.includes(entry)) found.push(entry);
      continue;
    }
    const otherLanguage = languages.find((other) =>
      getFunctionNames(document, other).has(name),
    );
    if (otherLanguage && !helperNames.has(name)) {
      const entry = getFunctionNames(document, otherLanguage).get(name);
      const right = getDisplayName(entry, language);
      if (!fixes.some((fix) => fix.from === name))
        fixes.push({
          from: name,
          to: right,
          reason: `${languageLabels[language]} names this function \`${right}\`.`,
        });
      if (!found.includes(entry)) found.push(entry);
      continue;
    }
    if (helperNames.has(name)) continue;
    const isMember = /(::|\.)\s*$/.test(masked.slice(0, match.index));
    if ((builtinNames[language] ?? []).includes(name)) continue;
    const builtin =
      !isMember &&
      (findClosest(name, builtinNames[language] ?? []) ??
        guesses[`builtin:${name}`]);
    if (builtin && !findClosest(name, [...functionNames.keys()])) {
      if (!fixes.some((fix) => fix.from === name))
        fixes.push({
          from: name,
          to: builtin,
          reason: `There is no function \`${name}\`; did you mean \`${builtin}\`?`,
        });
      continue;
    }
    const closest = findClosest(name, [...functionNames.keys()]);
    const guessed = document.entries.find(
      (entry) =>
        entry.kind === 'function' && entry.name === guesses[`:${name}`],
    );
    const right =
      closest ?? (guessed ? getDisplayName(guessed, language) : null);
    if (right && !fixes.some((fix) => fix.from === name)) {
      fixes.push({
        from: name,
        to: right,
        reason: `There is no function \`${name}\`; did you mean \`${right}\`?`,
      });
      const entry = functionNames.get(right);
      if (!found.includes(entry)) found.push(entry);
    } else if (
      !right &&
      !isMember &&
      !unknown.some((item) => item.name === name)
    )
      unknown.push({ kind: 'function', name, language });
  }

  const typePattern =
    language === 'rust'
      ? /\b([A-Z][A-Za-z0-9]*)\s*(\{|::)/g
      : language === 'js' || language === 'python'
        ? /(?::|\bas)\s*([A-Z][A-Za-z0-9]*)\b/g
        : null;
  if (typePattern) {
    for (const match of masked.matchAll(typePattern)) {
      const name = match[1];
      if (typeNames.has(name) || fixes.some((fix) => fix.from === name))
        continue;
      if (rustTypes.has(name)) continue;
      const closest =
        findClosest(name, [...typeNames]) ??
        (typeNames.has(guesses[`type:${name}`])
          ? guesses[`type:${name}`]
          : null);
      if (closest)
        fixes.push({
          from: name,
          to: closest,
          reason: `Quant·ion has no \`${name}\`; did you mean \`${closest}\`?`,
        });
      else if (
        /(Options?|Settings|Config|Configuration|Params|Parameters|Opts|Args|Results?)$/.test(
          name,
        ) &&
        !unknown.some((item) => item.name === name)
      )
        unknown.push({ kind: 'type', name });
    }
  }

  if (language === 'r') fixes.push(...findRConstructorFixes(code, document));
  const named = applyFixes(
    code,
    fixes.filter((fix) => /^[A-Za-z_]\w*$/.test(fix.from)),
  );
  if (language === 'rust')
    fixes.push(
      ...findMissingDefaults(named, document),
      ...findMissingArguments(named, found, document),
    );
  const state = { language, document, fixes, notes, guesses, unknown };
  const isBroken = notes.some((note) =>
    /never closed|straight quotes/.test(note),
  );
  if (!isBroken) {
    for (const entry of found) reviewOptions(named, entry, state);
    reviewOptionLiterals(named, state);
  }
  return {
    language,
    entries: found,
    fixes: fixes.map((fix) => ({
      ...fix,
      line: findFixLine(fix, code, masked, named),
    })),
    notes,
    unknown,
    values: isBroken
      ? []
      : found.flatMap((entry) =>
          readCodeValues(named, entry, language, document),
        ),
    code: applyFixes(code, fixes),
  };
}

export { detectLanguage, looksLikeCode, splitProse } from './code.js';
