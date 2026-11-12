export const convertLimits = { ion: 500, mzml: 1500 };

const formats = {
  ion: {
    label: '.ion',
    target: 'mzml',
    write: 'write_mzml',
    megabytes: convertLimits.ion,
  },
  mzml: {
    label: '.mzML',
    target: 'ion',
    write: 'write_ion',
    megabytes: convertLimits.mzml,
  },
};

const convertPattern =
  /\b(conver\w*|transform\w*|turn\w*|export\w*|change\w*|switch\w*|translat\w*|save\w*|make|get|need|want|to|into|as|from)\b|→|->/i;
const ion = String.raw`(^|[^\w·])\.?ion(ic)?\b`;
const ionPattern = new RegExp(ion, 'i');
const mzmlPattern = /mzml/i;
const toIonPatterns = [
  new RegExp(String.raw`mzml[\s\S]*(\b(to|into|as)\b|→|->)[\s\S]*?${ion}`, 'i'),
  new RegExp(String.raw`${ion}[\s\S]*\bfrom\b[\s\S]*mzml`, 'i'),
];
const toMzmlPatterns = [
  new RegExp(String.raw`${ion}[\s\S]*(\b(to|into|as)\b|→|->)[\s\S]*mzml`, 'i'),
  new RegExp(String.raw`mzml[\s\S]*\bfrom\b[\s\S]*?${ion}`, 'i'),
];

export function readConvertQuestion(question) {
  if (
    !convertPattern.test(question) ||
    !ionPattern.test(question) ||
    !mzmlPattern.test(question)
  )
    return null;
  const toIon = toIonPatterns.some((pattern) => pattern.test(question));
  const toMzml = toMzmlPatterns.some((pattern) => pattern.test(question));
  if (toIon !== toMzml) return toIon ? formats.mzml.write : formats.ion.write;
  return question.search(mzmlPattern) < question.search(ionPattern)
    ? formats.mzml.write
    : formats.ion.write;
}

export function readFileFormat(name) {
  const extension = name.toLowerCase().split('.').pop();
  if (extension === 'ion') return { source: 'ion', ...formats.ion };
  if (extension === 'mzml') return { source: 'mzml', ...formats.mzml };
  return null;
}

export function isTooBig(file, format) {
  return file.size > format.megabytes * 1000 * 1000;
}

function getConvertedName(name, format) {
  const base = name.replace(/\.[^.]+$/, '');
  return format.target === 'ion' ? `${base}.ion` : `${base}.mzML`;
}

export async function convertFile(file, options = {}) {
  const format = readFileFormat(file.name);
  if (!format) throw new Error('Only .ion and .mzML files can be converted.');
  if (isTooBig(file, format))
    throw new Error(
      `${format.label} files over ${format.megabytes} MB cannot be converted here. Convert it with the library instead.`,
    );
  const loaded = await import('quantion');
  const quantion = loaded.default ?? loaded;
  const bytes = new Uint8Array(await file.arrayBuffer());
  const sample =
    format.source === 'ion'
      ? await quantion.parseIon(bytes)
      : await quantion.parseMzml(bytes);
  try {
    const written =
      format.target === 'ion'
        ? await quantion.writeIon(sample, options)
        : await quantion.writeMzml(sample);
    const type =
      format.target === 'ion' ? 'application/octet-stream' : 'application/xml';
    return {
      blob: new Blob([written], { type }),
      name: getConvertedName(file.name, format),
    };
  } finally {
    sample.dispose?.();
  }
}
