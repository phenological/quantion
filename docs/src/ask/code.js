import { levenshteinDistance } from 'str-utilities';

const codeStart =
  /\b(use|let|const|var|import|from|library|fn|def)\b|[A-Za-z_][\w.]*\s*(<-|=(?!=))|[A-Za-z_]\w*\s*\(/;

const keywordShapes = {
  use: /^use\s+[\w:]+::/,
  import: /^import\s+(\{|\*|["']|\w+\s*(,|from\b|as\b|$))/m,
  from: /^from\s+[\w.]+\s+import\b/,
  const: /^const\s+[A-Za-z_]\w*\s*(:|=)/,
  let: /^let\s+(mut\s+)?[A-Za-z_]\w*\s*(:|=)/,
  def: /^def\s+\w+\s*\(/,
  fn: /^fn\s+\w+\s*[(<]/,
  library: /^library\s*\(/,
};

const realWords = new Set([
  'in',
  'if',
  'is',
  'as',
  'or',
  'and',
  'not',
  'for',
  'new',
  'del',
  'var',
  'mut',
  'pub',
  'try',
  'set',
  'def',
  'let',
  'use',
  'from',
]);

function isKeywordTypo(word, keyword) {
  if (word === keyword || word.length < 3 || realWords.has(word)) return false;
  if (levenshteinDistance(word, keyword) === 1) return true;
  return (
    word.length === keyword.length &&
    [...word].sort().join('') === [...keyword].sort().join('')
  );
}

export function findKeywordTypos(text) {
  const typos = [];
  for (const match of text.matchAll(/\b([A-Za-z]+)\b/g)) {
    const word = match[1];
    const rest = text.slice(match.index + word.length);
    const keyword = Object.keys(keywordShapes).find(
      (name) =>
        isKeywordTypo(word.toLowerCase(), name) &&
        keywordShapes[name].test(`${name}${rest}`),
    );
    if (keyword) typos.push({ index: match.index, word, keyword });
  }
  return typos;
}

const languageKeywords = {
  python: [
    'return',
    'else',
    'elif',
    'while',
    'lambda',
    'class',
    'def',
    'import',
    'from',
    'with',
    'yield',
    'async',
    'await',
    'break',
    'continue',
    'pass',
    'raise',
    'except',
    'finally',
    'True',
    'False',
    'None',
  ],
  js: [
    'return',
    'else',
    'while',
    'function',
    'async',
    'await',
    'const',
    'let',
    'import',
    'export',
    'class',
    'break',
    'continue',
    'throw',
    'typeof',
    'switch',
    'case',
    'default',
    'true',
    'false',
    'null',
    'undefined',
  ],
  rust: [
    'return',
    'else',
    'while',
    'struct',
    'impl',
    'match',
    'loop',
    'break',
    'continue',
    'enum',
    'trait',
    'mut',
    'pub',
    'use',
    'let',
    'where',
    'true',
    'false',
    'Some',
    'None',
  ],
  r: [
    'else',
    'repeat',
    'break',
    'next',
    'function',
    'while',
    'return',
    'TRUE',
    'FALSE',
    'NULL',
    'library',
  ],
};

export const builtinNames = {
  python: [
    'print',
    'len',
    'range',
    'list',
    'dict',
    'set',
    'str',
    'int',
    'float',
    'sum',
    'min',
    'max',
    'abs',
    'round',
    'sorted',
    'enumerate',
    'zip',
    'open',
    'isinstance',
    'tuple',
    'map',
    'filter',
    'any',
    'all',
  ],
  js: [
    'parseInt',
    'parseFloat',
    'Number',
    'String',
    'Array',
    'Object',
    'Boolean',
    'fetch',
    'require',
    'setTimeout',
    'Promise',
    'JSON',
    'Math',
    'console',
  ],
  rust: [
    'println',
    'print',
    'eprintln',
    'format',
    'vec',
    'panic',
    'assert',
    'assert_eq',
    'Some',
    'Ok',
    'Err',
    'write',
    'writeln',
  ],
  r: [
    'print',
    'cat',
    'paste',
    'paste0',
    'length',
    'nrow',
    'ncol',
    'seq',
    'seq_len',
    'seq_along',
    'rep',
    'sum',
    'mean',
    'max',
    'min',
    'abs',
    'sqrt',
    'round',
    'c',
    'list',
    'lapply',
    'sapply',
    'vapply',
    'unlist',
    'which',
    'names',
    'library',
    'stop',
    'warning',
    'message',
    'function',
    'while',
    'for',
    'if',
    'range',
    'sprintf',
    'nchar',
    'toupper',
    'tolower',
    'substr',
    'grepl',
    'gsub',
  ],
};

export const memberNames = {
  console: ['log', 'error', 'warn', 'info', 'table'],
  JSON: ['parse', 'stringify'],
  Object: ['keys', 'values', 'entries', 'assign', 'fromEntries'],
  Math: [
    'max',
    'min',
    'abs',
    'round',
    'floor',
    'ceil',
    'sqrt',
    'log',
    'exp',
    'pow',
  ],
  Promise: ['all', 'allSettled', 'race', 'resolve', 'reject'],
  Array: ['from', 'isArray', 'of'],
  rust: [
    'len',
    'push',
    'pop',
    'collect',
    'iter',
    'into_iter',
    'map',
    'filter',
    'unwrap',
    'expect',
    'clone',
    'to_string',
    'is_empty',
    'first',
    'last',
    'sum',
    'max',
    'min',
    'contains',
    'sort',
    'join',
    'split',
    'trim',
    'get',
    'insert',
  ],
};

export function findDefinedNames(masked) {
  const names = new Set();
  const patterns = [
    /\b(?:def|fn|function|class|struct|enum|impl|trait)\s+([A-Za-z_]\w*)/g,
    /\b([A-Za-z_][\w.]*)\s*(?:<-|=)\s*(?:function\b|lambda\b|(?:async\s*)?\([^)]*\)\s*=>)/g,
  ];
  for (const pattern of patterns)
    for (const match of masked.matchAll(pattern)) names.add(match[1]);
  return names;
}

export function findLooseKeywordTypos(masked, language, skip) {
  const keywords = languageKeywords[language] ?? [];
  const typos = [];
  for (const match of masked.matchAll(/\b([A-Za-z]+)\b/g)) {
    const word = match[1];
    if (keywords.includes(word) || skip.has(word)) continue;
    if (masked.match(new RegExp(`\\b${word}\\b`, 'g')).length > 1) continue;
    const before = masked.slice(0, match.index);
    const after = masked.slice(match.index + word.length);
    const isLabel =
      /^\s*:/.test(after) &&
      !(language === 'python' && /(^|\n)\s*$/.test(before));
    if (
      isLabel ||
      /(\.|::)\s*$/.test(before) ||
      /^\s*(\(|\[|\.|<-|=(?!=)|::)/.test(after)
    )
      continue;
    const keyword = keywords.find((name) =>
      isKeywordTypo(word.toLowerCase(), name.toLowerCase()),
    );
    if (keyword) typos.push({ index: match.index, word, keyword });
  }
  return typos;
}

export function findSyntaxHints(masked, language) {
  const hints = [];
  const lineAt = (position) => masked.slice(0, position).split('\n').length;
  let depth = 0;
  for (let position = 0; position < masked.length; position += 1) {
    if ('([{'.includes(masked[position])) depth += 1;
    else if (')]}'.includes(masked[position])) depth -= 1;
    const pair = /^(\d+(\.\d+)?)[ \t]+\d/.exec(masked.slice(position));
    if (depth > 0 && pair && !/[\w.]/.test(masked[position - 1] ?? ''))
      hints.push(
        `Line ${lineAt(position)}: a comma may be missing after \`${pair[1]}\`.`,
      );
  }
  if (language !== 'python')
    for (const match of masked.matchAll(/\bif\s*\(?\s*[\w.]+\s*=(?![=>])/g))
      hints.push(
        `Line ${lineAt(match.index)}: use \`==\` to compare, \`=\` sets a value.`,
      );
  if (language !== 'r') return hints;
  const rules = [
    [
      /\b(if|while|for)[ \t]+(?!\()/g,
      (match) => `R needs ( ) around the condition of \`${match[1]}\`.`,
    ],

    [/=>/g, () => 'R has no `=>`. Write `function(x) x * 2`.'],
    [/^[ \t]*[A-Za-z_][\w.]*[ \t]+[\d"']/gm, () => 'a `<-` may be missing.'],
    [
      /[+\-*/,][ \t]*$(?![\s\S]*\S)/g,
      () => 'the code ends with an operator, a value is missing.',
    ],
    [
      /\}[ \t]*\n[ \t]*else\b/g,
      () => 'in R, put `else` on the same line as `}`.',
      true,
    ],
    [/[!=]==/g, () => 'R compares with `==` and `!=`.'],
    [
      /^[ \t]*(const|let|var)[ \t]+[\w.]+[ \t]*<-/gm,
      (match) => `R has no \`${match[1]}\`. Write \`name <- value\`.`,
    ],
    [
      /\(([^()]*?)\b([A-Za-z_]\w*):[ \t]+[^:]/g,
      (match) => `R uses \`${match[2]} = value\` for arguments, not \`:\`.`,
    ],
  ];
  const depthAt = (position) =>
    [...masked.slice(0, position)].reduce(
      (total, character) =>
        total +
        ('([{'.includes(character) ? 1 : ')]}'.includes(character) ? -1 : 0),
      0,
    );
  for (const [pattern, message, topLevel] of rules)
    for (const match of masked.matchAll(pattern))
      if (!topLevel || depthAt(match.index + 1) === 0)
        hints.push(
          `Line ${lineAt(match.index + (match[0].startsWith('}') ? match[0].indexOf('else') : 0))}: ${message(match)}`,
        );
  return hints;
}

export function readCodeBlock(text) {
  const start = /[\/\\](code|check)\s*\{/.exec(text);
  if (!start) return null;
  const open = start.index + start[0].length - 1;
  const close = text.lastIndexOf('}');
  if (close <= open) return null;
  return {
    question: text.slice(0, start.index).trim(),
    code: text.slice(open + 1, close).trim(),
  };
}

export function splitProse(text) {
  const colon = /^\s*([A-Za-z][\w ,'’?!.-]*?)\s*:(?!:)\s*/.exec(text);
  if (
    colon &&
    colon[1].trim().split(/\s+/).length >= 2 &&
    !/[=(]/.test(colon[1]) &&
    !/\b(const|let|var|def|fn|pub|mut|async|await|return|class|struct)\b/.test(
      colon[1],
    )
  )
    return { prose: colon[1].trim(), code: text.slice(colon[0].length) };
  const paragraph = /^\s*([^\n]+(?:\n[^\n]+)*)\n\s*\n/.exec(text);
  if (
    paragraph &&
    paragraph[1].trim().split(/\s+/).length >= 2 &&
    !looksLikeCode(paragraph[1]) &&
    !/[;{}=]|^\s*(from|import|use|library)\b/.test(paragraph[1])
  )
    return {
      prose: paragraph[1].trim(),
      code: text.slice(paragraph[0].length),
    };
  const typo = findKeywordTypos(text)[0]?.index ?? -1;
  const keyword = text.search(codeStart);
  const start = typo >= 0 && (keyword < 0 || typo < keyword) ? typo : keyword;
  const before = start > 0 ? text.slice(0, start) : '';
  if (
    before.trim().split(/\s+/).length >= 3 &&
    /^[A-Za-z][\w ,'’?!.-]*$/.test(before.trim())
  )
    return { prose: before.trim(), code: text.slice(start) };
  return { prose: '', code: text };
}

export function looksLikeCode(text) {
  if (/<-|::|\bconst\b|\blet\b|\bimport\b|library\(/.test(text)) return true;
  if (/^\s*[A-Za-z_][\w.]*\s*=\s*(await\s+)?[A-Za-z_][\w.]*\s*\(/m.test(text))
    return true;
  return /\b[A-Za-z_]\w*\s*\([^)]*[,{=(]/.test(text);
}

export function detectLanguage(code, fallback) {
  if (
    /::|\.\.Default|\blet mut\b|&mut\b|\)\?;|\(&[a-z_]|\b[a-z_]+!\(|\b[A-Z]\w*\s*\{\s*[a-z_]\w*\s*:/.test(
      code,
    )
  )
    return 'rust';
  if (
    /\|[a-z_]\w*\|\s|\bVec<|\bSome\(|\.iter\(\)|\bimpl\s|\bstruct\s+\w+\s*\{|\bfn\s+\w+/.test(
      code,
    )
  )
    return 'rust';
  if (/<-|library\(|\b\w+_options\(/.test(code)) return 'r';
  if (/^\s*(from [\w.]+ import|import [\w.]+\s*$)|\bdef\s+\w+\(/m.test(code))
    return 'python';
  if (/\b(const|await)\b|=>|\bimport\s*\{/.test(code)) return 'js';
  if (
    /\{\s*["'][\w-]+["']\s*:|^\s*from \w+ import|\bTrue\b|\bFalse\b|\bNone\b/m.test(
      code,
    )
  )
    return 'python';
  if (
    /\b(const|let|await)\b|\{\s*[A-Za-z_]\w*\s*:|\b[a-z]+[A-Z]\w*\s*\(/.test(
      code,
    )
  )
    return 'js';
  return fallback;
}

const openers = { '(': ')', '[': ']', '{': '}' };

const closers = { ')': '(', ']': '[', '}': '{' };

function readString(code, start, language) {
  const quote = code[start];
  const triple =
    language === 'python' && code.slice(start, start + 3) === quote.repeat(3);
  const end = triple ? quote.repeat(3) : quote;
  const multiline =
    triple || quote === '`' || language === 'rust' || language === 'r';
  for (
    let position = start + end.length;
    position < code.length;
    position += 1
  ) {
    if (code[position] === '\\') position += 1;
    else if (code.startsWith(end, position)) return position + end.length - 1;
    else if (code[position] === '\n' && !multiline) return -1;
  }
  return -1;
}

function isRustLifetime(code, position) {
  return !/^'(\\.|[^\\'])'/.test(code.slice(position, position + 4));
}

function readSpecial(code, position, language) {
  if (language === 'rust' && !/\w/.test(code[position - 1] ?? '')) {
    const raw = /^r(#*)"/.exec(code.slice(position, position + 20));
    if (raw) {
      const end = code.indexOf(`"${raw[1]}`, position + raw[0].length);
      return end < 0 ? -1 : end + raw[1].length;
    }
  }
  if (
    language === 'js' &&
    code[position] === '/' &&
    !'/*'.includes(code[position + 1])
  ) {
    const before = code.slice(0, position).trimEnd();
    if (before !== '' && !/[=(,:[!&|?{};]$|\breturn$/.test(before)) return null;
    let inClass = false;
    for (let index = position + 1; index < code.length; index += 1) {
      const character = code[index];
      if (character === '\\') index += 1;
      else if (character === '\n') return null;
      else if (character === '[') inClass = true;
      else if (character === ']') inClass = false;
      else if (character === '/' && !inClass) return index;
    }
  }
  return null;
}

function scanCode(code, language) {
  const characters = [...code];
  const blank = (from, to) => {
    for (let position = from; position < to; position += 1)
      if (characters[position] !== '\n') characters[position] = ' ';
  };
  const hashComments = language === 'python' || language === 'r';
  let unclosed = null;
  for (let position = 0; position < code.length; position += 1) {
    const character = code[position];
    const lineComment =
      (hashComments && character === '#') ||
      (!hashComments && code.startsWith('//', position));
    const blockComment = !hashComments && code.startsWith('/*', position);
    if (lineComment || blockComment) {
      const end = lineComment
        ? code.indexOf('\n', position)
        : code.indexOf('*/', position + 2) + 1;
      blank(position, end <= 0 ? code.length : end + (lineComment ? 0 : 1));
      if (end <= 0) break;
      position = end;
      continue;
    }
    let end = readSpecial(code, position, language);
    if (end === null && '"\'`'.includes(character)) {
      if (character === '`' && language !== 'js') continue;
      if (
        character === "'" &&
        language === 'rust' &&
        isRustLifetime(code, position)
      )
        continue;
      end = readString(code, position, language);
    }
    if (end === null) continue;
    if (end < 0) {
      unclosed = { position, character: character === 'r' ? '"' : character };
      break;
    }
    blank(position + 1, end);
    position = end;
  }
  return { masked: characters.join(''), unclosed };
}

export function maskCode(code, language) {
  return scanCode(code, language).masked;
}

export function checkBrackets(code, language) {
  const problems = [];
  const lineAt = (position) => code.slice(0, position).split('\n').length;
  if (language === 'rust') {
    for (const match of code.matchAll(
      /\b(println|print|eprintln|format|panic)!\(\s*(?=[^"\s)])/g,
    ))
      problems.push(
        `Line ${lineAt(match.index)}: \`${match[1]}!\` needs its text in quotes, for example \`${match[1]}!("{peaks:?}")\`.`,
      );
  }
  const { masked, unclosed } = scanCode(code, language);
  const curly = masked.search(/[“”‘’]/);
  if (curly >= 0 && (!unclosed || curly < unclosed.position))
    return [
      ...problems,
      `Line ${lineAt(curly)}: use straight quotes (" or ') instead of ${masked[curly]}.`,
    ];
  if (unclosed)
    return [
      ...problems,
      `Line ${lineAt(unclosed.position)}: the ${unclosed.character} that starts here is never closed.`,
    ];
  const stack = [];
  for (let position = 0; position < masked.length; position += 1) {
    const character = masked[position];
    if (openers[character]) stack.push({ character, line: lineAt(position) });
    if (!closers[character]) continue;
    const last = stack.pop();
    if (!last)
      return [
        ...problems,
        `Line ${lineAt(position)}: extra \`${character}\` with no \`${closers[character]}\` before it.`,
      ];
    if (last.character !== closers[character])
      return [
        ...problems,
        `Line ${lineAt(position)}: \`${character}\` closes the \`${last.character}\` from line ${last.line}, which needs \`${openers[last.character]}\`.`,
      ];
  }
  for (const open of stack.reverse())
    problems.push(
      `Line ${open.line}: \`${open.character}\` is never closed, a \`${openers[open.character]}\` is missing.`,
    );
  return problems;
}

export function findClose(code, open) {
  const closer = openers[code[open]];
  let depth = 0;
  for (let position = open; position < code.length; position += 1) {
    if (code[position] === code[open]) depth += 1;
    else if (code[position] === closer && --depth === 0) return position;
  }
  return -1;
}

export function readTopLevel(body) {
  let depth = 0;
  let text = '';
  for (const character of body) {
    if ('{(['.includes(character)) depth += 1;
    else if ('})]'.includes(character)) depth -= 1;
    text += depth === 0 ? character : ' ';
  }
  return text;
}
