const languageNames = {
  python: 'Python',
  js: 'JavaScript or TypeScript',
  rust: 'Rust',
};

async function loadParser(language) {
  if (language === 'python') return (await import('@lezer/python')).parser;
  if (language === 'js')
    return (await import('@lezer/javascript')).parser.configure({
      dialect: 'ts',
    });
  if (language === 'rust') return (await import('@lezer/rust')).parser;
  return null;
}

function removeIndent(code) {
  const lines = code.split('\n');
  const indents = lines
    .filter((line) => line.trim())
    .map((line) => line.match(/^[ \t]*/)[0].length);
  const common = indents.length > 0 ? Math.min(...indents) : 0;
  return lines.map((line) => line.slice(common)).join('\n');
}

function wrapCode(code, language) {
  if (language === 'rust' && !/\bfn\s+\w+/.test(code))
    return { text: `fn main() {\n${code}\n}`, shift: 1 };
  if (language === 'python') return { text: removeIndent(code), shift: 0 };
  return { text: code, shift: 0 };
}

export async function findSyntaxError(code, language) {
  const parser = await loadParser(language);
  if (!parser) return null;
  const { text, shift } = wrapCode(code, language);
  let error = null;
  parser.parse(text).iterate({
    enter(node) {
      if (error) return false;
      if (node.type.isError) error = node.from;
    },
  });
  if (error === null) return null;
  const lines = text.split('\n');
  const index = text.slice(0, error).split('\n').length - 1;
  const line = Math.min(
    Math.max(index - shift, 0),
    code.split('\n').length - 1,
  );
  const lineStart = text.lastIndexOf('\n', error - 1) + 1;
  const before =
    lines
      .slice(0, index)
      .filter((row) => row.trim())
      .pop()
      ?.trim() ?? '';
  if (
    language !== 'python' &&
    !text.slice(lineStart, error).trim() &&
    before &&
    !/[;{},([]$/.test(before)
  ) {
    return `Line ${line}: a \`;\` may be missing at the end of the line.`;
  }
  const shown = text
    .slice(Math.max(lineStart, error - 20), error + 20)
    .split('\n')[0]
    .trim();
  return `Line ${line + 1}: this is not valid ${languageNames[language]}${shown ? `, near \`${shown}\`` : ''}.`;
}
