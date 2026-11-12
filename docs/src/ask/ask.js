import { getDisplayName } from '../names.js';
import { isCommand, runCommand } from './commands.js';
import { readConvertQuestion } from './convert.js';
import { findSyntaxError } from './syntax.js';
import {
  findVariant,
  getFunctionFollowUps,
  guideItems,
  hasVariants,
  listVariants,
  starters,
} from './guide.js';
import { limits, models } from './models.js';
import {
  getOptionsType,
  getPlainName,
  listOptionFunctions,
  listValueKeys,
} from './snippet.js';
import { findEntry, findGroup } from '../state/documentationState.js';
import { readCodeBlock } from './code.js';
import faqItems from './faq.json' with { type: 'json' };
import { looksLikeCode, reviewCode, splitProse } from './review.js';
import { createValueReader } from './values.js';

function splitSentences(text) {
  return text.split(/(?<=\.)\s+/).filter((sentence) => sentence.length > 3);
}

function trimAnswer(text) {
  const clean = text.replace(/\s+/g, ' ').trim();
  if (clean.length <= limits.answer) return clean;
  const head = clean.slice(0, limits.answer);
  const sentenceEnd = head.lastIndexOf('. ');
  if (sentenceEnd > limits.answer / 3) return head.slice(0, sentenceEnd + 1);
  return `${head.slice(0, head.lastIndexOf(' '))}…`;
}

export function createClassifier({ labels, scale, dimension }, buffer) {
  const numbers = new Float32Array(buffer);
  const bias = numbers.subarray(labels.length * dimension);
  return (vector) => {
    const scores = labels.map((label, row) => {
      let score = 0;
      for (let column = 0; column < dimension; column += 1)
        score += numbers[row * dimension + column] * vector[column];
      return score * scale + bias[row];
    });
    const largest = Math.max(...scores);
    const exponents = scores.map((score) => Math.exp(score - largest));
    const total = exponents.reduce((sum, value) => sum + value, 0);
    return labels
      .map((id, row) => ({ id, probability: exponents[row] / total }))
      .sort((left, right) => right.probability - left.probability);
  };
}

function readAnswerId(id) {
  const split = id.indexOf(':');
  if (split < 0) return { route: id, target: '' };
  return { route: id.slice(0, split), target: id.slice(split + 1) };
}

const genericWords = new Set(['get', 'set', 'the', 'and', 'options', 'option']);

function splitWords(name) {
  return name
    .replace(/([a-z0-9])([A-Z])/g, '$1_$2')
    .toLowerCase()
    .split(/[_\W]+/)
    .filter((word) => word.length >= 3 && !genericWords.has(word));
}

function sharesWord(wrong, right) {
  const rightWords = splitWords(right);
  return splitWords(wrong).some((word) =>
    rightWords.some(
      (other) => other.startsWith(word) || word.startsWith(other),
    ),
  );
}

export function createCodeScorer(meta, buffer) {
  const classify = createClassifier(meta, buffer);
  return (vector, functions, options) =>
    new Map(
      classify([
        ...vector,
        ...meta.functions.map((name) => (functions.includes(name) ? 1 : 0)),
        ...meta.options.map((name) => (options.includes(name) ? 1 : 0)),
      ]).map(({ id, probability }) => [id, Math.log(probability)]),
    );
}

export function createNameFixer(classify, encode, document) {
  const functions = new Set(
    document.entries
      .filter((entry) => entry.kind === 'function')
      .map((entry) => entry.name),
  );
  const types = new Set(
    document.entries
      .filter((entry) => entry.kind !== 'function')
      .map((entry) => entry.name),
  );
  const optionsOwners = new Map();
  for (const entry of document.entries) {
    const type = entry.kind === 'function' && getOptionsType(entry, document);
    if (type) optionsOwners.set(type.name, entry.name);
  }
  const toFunction = (id) => {
    const owner = id.includes('@') ? id.split('@')[1] : id;
    if (functions.has(owner)) return owner;
    return optionsOwners.get(owner) ?? null;
  };
  return async (unknown) => {
    const guesses = {};
    for (const item of unknown) {
      const ranked = classify(await encode(item.name));
      const best = ranked.find(({ id }) => {
        if (id === 'none') return true;
        if (item.kind === 'function') {
          const name = toFunction(id);
          return Boolean(name) && sharesWord(item.name, name);
        }
        if (item.kind === 'type') return types.has(id);
        const [field, owner] = id.split('@');
        return item.owners.includes(owner) && item.fields.includes(field);
      });
      const key =
        item.kind === 'function'
          ? `:${item.name}`
          : item.kind === 'type'
            ? `type:${item.name}`
            : `${item.function}:${item.name}`;
      if (item.kind === 'function' && item.language && best?.id === 'none') {
        const scope = 'builtin';
        const prefix = `${scope}:${item.language}:`;
        const scoped = ranked.filter(
          ({ id }) => id.startsWith(prefix) || id === 'none',
        );
        const total = scoped.reduce(
          (sum, { probability }) => sum + probability,
          0,
        );
        if (
          scoped[0]?.id !== 'none' &&
          scoped[0]?.probability / total >= limits.builtinGuess
        )
          guesses[`${scope}:${item.name}`] = scoped[0].id.slice(prefix.length);
      }
      if (best && best.id !== 'none')
        guesses[key] =
          item.kind === 'function'
            ? toFunction(best.id)
            : best.id.split('@')[0];
    }
    return guesses;
  };
}

function findExampleEntry(entries, language) {
  for (const entry of entries) {
    if (entry.languages[language]?.example) return entry;
  }
  return null;
}

const localRuntimes = new WeakSet();

function useLocalRuntime(wasm, runtimeFiles) {
  if (!wasm || !runtimeFiles || localRuntimes.has(wasm)) return;
  const current =
    typeof wasm.wasmPaths === 'object' && wasm.wasmPaths
      ? Object.values(wasm.wasmPaths)
      : [];
  if (
    current.some(
      (address) =>
        String(address).startsWith('blob:') ||
        Object.values(runtimeFiles).includes(address),
    )
  ) {
    localRuntimes.add(wasm);
    return;
  }
  const paths =
    typeof wasm.wasmPaths === 'object' && wasm.wasmPaths
      ? wasm.wasmPaths
      : {
          mjs: 'ort-wasm-simd-threaded.asyncify.mjs',
          wasm: 'ort-wasm-simd-threaded.asyncify.wasm',
        };
  const local = {};
  for (const [kind, address] of Object.entries(paths)) {
    const name = String(address).split('/').pop();
    if (!runtimeFiles[name])
      throw new Error(`No local ONNX runtime file ${name}`);
    local[kind] = runtimeFiles[name];
  }
  wasm.wasmPaths = local;
  localRuntimes.add(wasm);
}

async function loadTransformers(options) {
  if (!options.modelFolder)
    throw new Error('createEncoder needs a modelFolder');
  const transformers = await import('@huggingface/transformers');
  transformers.env.allowLocalModels = true;
  transformers.env.allowRemoteModels = false;
  transformers.env.localModelPath = options.modelFolder;
  useLocalRuntime(transformers.env.backends?.onnx?.wasm, options.runtimeFiles);
  return transformers;
}

export async function createEncoder(options = {}) {
  const transformers = await loadTransformers(options);
  const extractor = await transformers.pipeline(
    'feature-extraction',
    models.encoder.name,
    {
      dtype: models.encoder.dtype,
      device: options.device,
      session_options: options.sessionOptions,
      progress_callback: options.onProgress,
    },
  );
  return async (text) => {
    const output = await extractor(text, {
      pooling: models.encoder.pooling,
      normalize: true,
    });
    return Array.from(output.data);
  };
}

function formatAnswerValue(value) {
  if (typeof value === 'string') return `"${value}"`;
  return String(value);
}

function findValueItem(entry, document, name) {
  for (const key of listValueKeys(entry, document)) {
    if (key.item.languages.python.name === name) return key.item;
  }
  return null;
}

function describeValues(entry, document, language, values) {
  const parts = [];
  for (const [name, value] of Object.entries(values)) {
    if (name === 'function') continue;
    const item = findValueItem(entry, document, name);
    parts.push(
      `\`${item.languages[language].name}\` = ${formatAnswerValue(value)}`,
    );
  }
  return parts.join(' and ');
}

export function listUnderstood(result, document, language) {
  const entry = result.entries[0];
  const chips = [
    {
      kind: 'function',
      entryName: entry.name,
      label: getDisplayName(entry, language),
    },
  ];
  for (const [name, value] of Object.entries(result.values ?? {})) {
    if (name === 'function') continue;
    const item = findValueItem(entry, document, name);
    chips.push({
      kind: 'value',
      owner: entry.name,
      name,
      label: `${item.languages[language].name} = ${formatAnswerValue(value)}`,
      value,
    });
  }
  for (const [ownerName, values] of Object.entries(
    result.templateValues ?? {},
  )) {
    const owner = findEntry(document, ownerName);
    for (const [name, value] of Object.entries(values)) {
      const item = findValueItem(owner, document, name);
      chips.push({
        kind: 'value',
        owner: ownerName,
        name,
        label: `${item.languages[language].name} = ${formatAnswerValue(value)} (${getDisplayName(owner, language)})`,
        value,
      });
    }
  }
  return chips;
}

export function writeAnswer(result, document, language) {
  const entry = result.entries[0];
  const functionName = `\`${getDisplayName(entry, language)}\``;
  const own = describeValues(entry, document, language, result.values);
  const helpers = [];
  for (const [name, values] of Object.entries(result.templateValues ?? {})) {
    const helper = document.entries.find((item) => item.name === name);
    helpers.push(
      `\`${getDisplayName(helper, language)}\` with ${describeValues(helper, document, language, values)}`,
    );
  }
  let text;
  if (result.variant?.refused) text = result.variant.refused;
  else if (own) text = `Use ${functionName} with ${own}.`;
  else if (result.variant)
    text = `Use ${functionName} with the ${result.variant.label.toLowerCase().replace(/^from (a |an )?/, '')}: ${splitSentences(entry.summary)[0]}`;
  else if (helpers.length === 0 && result.mentioned) {
    text = `Use ${functionName} and set its option \`${result.mentioned.languages[language].name}\`: ${result.mentioned.meaning}`;
    const address = result.mentioned.languages[language].default?.match(
      /^"(https?:\/\/[^"]+)"$/,
    )?.[1];
    if (address) text += ` By default it is ${address}.`;
  } else text = `Use ${functionName}: ${entry.summary}`;
  if (helpers.length > 0)
    text += ` In the template, use ${helpers.join(' and ')}.`;
  else if (result.helperMention) {
    const { owner, item } = result.helperMention;
    text += ` The \`${item.languages[language].name}\` goes in \`${getDisplayName(owner, language)}\`: ${item.meaning}`;
  }
  return trimAnswer(text);
}

export function writePlacement(placement, language) {
  const { entry, owner, item } = placement;
  const text = `Put it in \`${item.languages[language].name}\` of \`${getDisplayName(owner, language)}\`: ${item.meaning}`;
  const example = entry.languages[language]?.example ?? '';
  const line = example
    .split('\n')
    .find((row) => row.includes(`${getDisplayName(owner, language)}(`));
  return { text, line: line?.trim() ?? null, entry };
}

function getTemplateFunctions(entry, document) {
  const example = entry?.languages.python?.example ?? '';
  const found = [];
  for (const candidate of document.entries) {
    if (
      candidate.kind !== 'function' ||
      candidate === entry ||
      candidate.name === 'dispose'
    )
      continue;
    if (new RegExp(`\\b${candidate.name}\\(`).test(example))
      found.push(candidate);
  }
  return found;
}

const numberValuePattern = /\d+(\.\d+)?/;

function isCodeForm(entry, text) {
  if (!entry.name.includes('_'))
    return new RegExp(`\\b${entry.name}\\(|\\.${entry.name}\\b`, 'i').test(
      text,
    );
  const snake = entry.name.replace(/s$/, '');
  const camel = getDisplayName(entry, 'js').replace(/s$/, '');
  return new RegExp(`(^|[^\\w.])(${snake}|${camel})s?(\\(|\\b)`, 'i').test(
    text,
  );
}

function makeLink(entry, language) {
  if (!entry) return { entryName: 'overview', label: 'Overview' };
  return { entryName: entry.name, label: getDisplayName(entry, language) };
}

function getUsageFollowUps(entry, variant) {
  if (hasVariants(entry.name)) {
    return listVariants(entry.name)
      .filter((item) => item.form !== variant?.form)
      .slice(0, 2)
      .map((item) => ({
        label: item.label,
        question: `${item.label} with ${entry.name}`,
      }));
  }
  return getFunctionFollowUps(entry.name).slice(0, 2);
}

export function getStarters() {
  return starters;
}

const protonMass = 1.007276;

function readIonMz(question) {
  const text = question.toLowerCase();
  const negative = /\bneg(ative)?\b|\[m\s*-\s*h\]|esi\s*-/.test(text);
  const positive = /\bpos(itive)?\b|\[m\s*\+\s*h\]|esi\s*\+/.test(text);
  if (negative === positive) return null;
  if (!/\b(mass|monoisotopic|neutral|molecular weight|mw)\b/.test(text))
    return null;
  const numbers = (text.match(/\d+\.\d+|\d{2,}/g) ?? [])
    .map(Number)
    .filter((value) => value >= 20 && value <= 5000);
  if (numbers.length !== 1) return null;
  const mass = numbers[0];
  const decimals = Math.max(4, (String(mass).split('.')[1] ?? '').length);
  const mz = (negative ? mass - protonMass : mass + protonMass).toFixed(
    decimals,
  );
  return negative
    ? {
        mode: 'negative',
        name: 'deprotonated molecule',
        formula: '[M−H]−',
        sign: '−',
        mass,
        mz,
      }
    : {
        mode: 'positive',
        name: 'protonated molecule',
        formula: '[M+H]+',
        sign: '+',
        mass,
        mz,
      };
}

function findItem(entry, name, document) {
  const items = [
    ...(entry.arguments ?? []),
    ...(entry.fields ?? []),
    ...(getOptionsType(entry, document)?.fields ?? []),
  ];
  return (
    items.find((item) => getPlainName(item) === name) ??
    items.find(
      (item) => getPlainName(item).toLowerCase() === name.toLowerCase(),
    ) ??
    null
  );
}

function getOwnerName(target) {
  return target.includes('@') ? target.split('@')[1] : target;
}

function rateProbability(probability) {
  if (probability >= limits.highProbability) return 'high';
  if (probability >= limits.lowProbability) return 'medium';
  return 'low';
}

const vagueFaqs = ['error-unclear', 'vague-wrong'];
const problemPattern =
  /\b(not working|doesn'?t work|does not work|broken|wrong|weird|strange|odd|problem|issue|help|fails?|failing|bad)\b/i;
const rerankWeights = {
  candidates: 20,
  fromLayer: 3,
  features: [
    0.747, 0.541, 0.568, 0.521, 0.014, 0.307, -0.008, 0.174, -0.053, 1.726,
  ],
};

function findFaq(id) {
  return faqItems.find((item) => item.id === id) ?? null;
}

function faqChip(item, name) {
  const label = name ? item.short.replaceAll('{fn}', name) : item.question;
  return { label, question: label, faqId: item.id };
}

function relatedFaqs(item) {
  return item.related
    .map(findFaq)
    .filter(Boolean)
    .map((other) => faqChip(other));
}

export function rerankFeatures(
  candidate,
  focusOf,
  codeFunctions,
  values,
  layerScores = null,
) {
  const layer = layerScores
    ? (layerScores.get(candidate.id) ?? layerScores.get('other') ?? 0)
    : 0;
  return [...codeFeatures(candidate, focusOf, codeFunctions, values), layer];
}

function codeFeatures(candidate, focusOf, codeFunctions, values) {
  const { route, target } = readAnswerId(candidate.id);
  const item = route === 'faq' ? findFaq(target) : null;
  const logProbability = Math.log(candidate.probability);
  if (route === 'notfaq') return [logProbability, 0, 0, 0, 0, 0, 0, 0, 1];
  if (!item) {
    const about = target.split('@').pop();
    const inCode = codeFunctions.includes(about) ? 1 : 0;
    return [logProbability, inCode, inCode, inCode, 0, 0, 0, 1, 0];
  }
  if (vagueFaqs.includes(item.id))
    return [logProbability, 0, 0, 0, 0, 0, 1, 0, 0];
  const focus = focusOf(item);
  const shared = focus.filter((name) => codeFunctions.includes(name)).length;
  const uses = item.uses.some((name) => codeFunctions.includes(name));
  const options = item.options.filter((option) =>
    values.some((value) => value.name === option.name),
  ).length;
  return [
    logProbability,
    shared / (focus.length || 1),
    shared > 0 ? 1 : 0,
    uses ? 1 : 0,
    options,
    options > 0 ? 1 : 0,
    0,
    0,
    0,
  ];
}

function rerankFaqs(
  ranked,
  focusOf,
  codeFunctions,
  values,
  layerScores = null,
  weights = rerankWeights,
) {
  const pool = ranked.slice(0, weights.candidates);
  const fromLayer = [...(layerScores ?? [])]
    .filter(([id]) => id !== 'other')
    .sort((left, right) => right[1] - left[1])
    .slice(0, weights.fromLayer)
    .map(([id]) => ranked.find((candidate) => candidate.id === id))
    .filter((candidate) => candidate && !pool.includes(candidate));
  const scored = [...pool, ...fromLayer].map((candidate) => ({
    ...candidate,
    score: rerankFeatures(
      candidate,
      focusOf,
      codeFunctions,
      values,
      layerScores,
    ).reduce((sum, value, index) => sum + value * weights.features[index], 0),
  }));
  const largest = Math.max(...scored.map((candidate) => candidate.score));
  const total = scored.reduce(
    (sum, candidate) => sum + Math.exp(candidate.score - largest),
    0,
  );
  return scored
    .map((candidate) => ({
      ...candidate,
      confidence: Math.exp(candidate.score - largest) / total,
    }))
    .sort((left, right) => right.score - left.score);
}

function readEnum(value) {
  return String(value).split(/[.:]/).pop().replace(/"/g, '').toUpperCase();
}

function matchesWhen(rule, values, item) {
  return rule.terms.every((term) => {
    const set = values.find((value) => value.name === term.name);
    if (term.op === 'set') return Boolean(set);
    const current =
      set?.value ??
      item.options.find((option) => option.name === term.name)?.default;
    const same = readEnum(current ?? '') === readEnum(term.value);
    return term.op === '=' ? same : !same;
  });
}

function fillValues(text, values, item) {
  let filled = text.replace(/\{(\w+)\}/g, (whole, name) => {
    const set = values.find((value) => value.name === name);
    return (
      set?.value ??
      item.options.find((option) => option.name === name)?.default ??
      whole
    );
  });
  for (const { name, value } of values) {
    const mark = `\`${name}\``;
    if (!filled.includes(mark) || filled.includes(`${mark} (you set`)) continue;
    const withDefault = new RegExp(
      `${mark} \\((default [^,)]+|off by default)`,
    );
    filled = withDefault.test(filled)
      ? filled.replace(withDefault, `${mark} (you set ${value}, $1`)
      : filled.replace(
          new RegExp(`${mark}(?! \\()`),
          `${mark} (you set ${value})`,
        );
  }
  return filled;
}

function mentionedNames(text) {
  return [...text.matchAll(/`(\w+)`/g)]
    .map((match) => match[1])
    .filter((name) => name !== 'calculate_eic');
}

function mentionsSet(text, values) {
  return values.some((value) => text.includes(`\`${value.name}\``));
}

function pickFaqNotes(item, question, codeFunctions) {
  const squashed = question.toLowerCase().replace(/[_\s-]/g, '');
  return Object.entries(item.notes)
    .filter(
      ([name]) =>
        codeFunctions.includes(name) ||
        squashed.includes(name.toLowerCase().replace(/_/g, '')),
    )
    .map(([, note]) => note);
}

const codeRequestPattern =
  /\b(code|snippet|example|script|template|sample code|show me|write (me )?(the|a|some)?)\b/i;

const ownCodePattern =
  /\b(my|this|the following|attached|pasted|same) code\b|\bcode (below|above|attached|here)\b|\bsee (the |my )?code\b/gi;

export function asksForCode(question) {
  return codeRequestPattern.test(question.replace(ownCodePattern, ' '));
}

function findFileNote(question, entries) {
  if (!/\bmz ?ml\b/i.test(question)) return null;
  const takesFile = entries.some((entry) =>
    (entry.arguments ?? []).some((argument) =>
      /SampleFile/.test(argument.languages.rust?.type ?? ''),
    ),
  );
  if (!takesFile) return null;
  return guideItems.find((item) => item.id === 'open-file')?.answer ?? null;
}

export function createAsker({
  encode,
  classify,
  fixNames,
  scoreCode,
  document,
}) {
  const readValues = encode ? createValueReader(encode) : null;

  async function finishUsage(asked, entries, confidence, extra) {
    const leading = entries[0];
    const variant = findVariant(asked, leading.name);
    const read = readValues
      ? await readValues(
          asked,
          leading,
          document,
          getTemplateFunctions(leading, document),
        )
      : { values: { function: leading.name }, templateValues: {} };
    return {
      ...extra,
      reason: { route: 'usage', confidence },
      fileNote: findFileNote(asked, [leading]),
      values: read.values,
      templateValues: read.templateValues,
      helperMention: read.helperMention,
      mentioned: read.mentioned,
      entries,
      variant,
      link: makeLink(leading, extra.language),
      followUps: getUsageFollowUps(leading, variant),
      alternatives: entries.slice(1).map((entry) => ({
        entryName: entry.name,
        label: getDisplayName(entry, extra.language),
      })),
    };
  }

  function guideResult(item, confidence, language, extra = {}) {
    const entries = item.functions
      .map((name) => findEntry(document, name))
      .filter(Boolean);
    const fileNote = findFileNote(extra.question ?? '', entries);
    const linked =
      findEntry(document, item.link) ?? findGroup(document, item.link);
    const link = linked
      ? {
          entryName: linked.name,
          label: linked.kind ? getDisplayName(linked, language) : linked.name,
        }
      : { entryName: 'overview', label: 'Overview' };
    const amount = extra.question?.match(numberValuePattern)?.[0];
    const fill = (text) =>
      amount
        ? text.replaceAll('{value}', amount)
        : text.replaceAll(' {value}', '').replaceAll('{value}', '');
    const followUps = item.followUps.map((followUp) => ({
      label: fill(followUp.label),
      question: followUp.question ? fill(followUp.question) : undefined,
    }));
    return {
      ...extra,
      reason: { route: 'guide', confidence },
      guide: item,
      fileNote,
      answer: fill(amount ? item.answer : (item.answerWithout ?? item.answer)),
      entries,
      values: null,
      link,
      followUps,
    };
  }

  function notSureResult(extra = {}) {
    return {
      ...extra,
      reason: { route: 'not-sure', confidence: 'low' },
      entries: [],
      values: null,
      followUps: [],
    };
  }

  function refusedResult(extra = {}) {
    return {
      ...extra,
      reason: { route: 'refused', confidence: 'low' },
      unknown: true,
      entries: [],
      values: null,
      followUps: starters,
    };
  }

  function listResult(option, entries, language) {
    const followUps = entries.map((entry) => ({
      label: getDisplayName(entry, language),
      question: `How do I use ${entry.name} with ${option}?`,
    }));
    return {
      reason: { route: 'list', confidence: 'high' },
      list: { option, entries },
      option,
      entries,
      values: null,
      followUps,
    };
  }

  function findFunction(target) {
    const owner = findEntry(document, getOwnerName(target));
    if (!owner) return null;
    if (owner.kind === 'function') return owner;
    return (
      document.entries.find(
        (entry) =>
          entry.kind === 'function' &&
          getOptionsType(entry, document) === owner,
      ) ?? null
    );
  }

  function listFunctions(ranked) {
    const found = [];
    for (const { id } of ranked) {
      const entry = findFunction(readAnswerId(id).target);
      if (entry && !found.includes(entry)) found.push(entry);
      if (found.length === limits.entries) break;
    }
    return found;
  }

  async function answerFromId(
    id,
    probability,
    ranked,
    asked,
    questionVector,
    language,
    pasted = false,
  ) {
    const { route, target } = readAnswerId(id);
    const confidence = rateProbability(probability);
    const extra = { questionVector, probability, answerId: id };
    if (route === 'refuse') return refusedResult(extra);
    if (route === 'notfaq') return notSureResult(extra);
    if (route === 'faq') {
      const item = faqItems.find((candidate) => candidate.id === target);
      if (!item) return null;
      const links = findFaqLinks(item, namedFunctions(asked), language);
      return {
        ...extra,
        reason: { route: 'faq', confidence },
        faq: {
          id: item.id,
          note: item.intro,
          ideas: item.ideas,
          notes: pickFaqNotes(item, asked, []),
          owners: [...item.functions, ...item.owners],
          route,
          target: item.functions[0] ?? '',
        },
        entries: [],
        values: null,
        link: links[0] ?? null,
        links,
        followUps: relatedFaqs(item),
      };
    }
    if (route === 'unsupported') {
      return {
        ...extra,
        reason: { route: 'unsupported', confidence },
        faq: { note: 'Quant·ion does not do that.', route, target: '' },
        entries: [],
        values: null,
        link: null,
        followUps: starters,
      };
    }
    if (route === 'guide') {
      const item = guideItems.find((candidate) => candidate.id === target);
      return item
        ? guideResult(item, confidence, language, { ...extra, question: asked })
        : null;
    }
    if (route === 'list') {
      const entries = listOptionFunctions(target, document);
      return entries.length > 0
        ? { ...extra, ...listResult(target, entries, language) }
        : null;
    }
    if (
      route === 'definition' &&
      !target.includes('@') &&
      findGroup(document, target)
    ) {
      return {
        ...extra,
        reason: { route: 'definition', confidence },
        explanation: { type: 'group', group: findGroup(document, target) },
        entries: [],
        values: null,
        followUps: [],
      };
    }
    const owner = findEntry(document, getOwnerName(target));
    if (!owner) return null;
    if (route === 'usage' && owner.kind === 'function') {
      const entries = [
        owner,
        ...listFunctions(ranked).filter((entry) => entry !== owner),
      ].slice(0, limits.entries);
      return finishUsage(asked, entries, confidence, {
        ...extra,
        language,
        example: findExampleEntry(entries, language),
      });
    }
    if (
      route === 'placement' &&
      owner.kind === 'function' &&
      !pasted &&
      (asksForCode(asked) || (asked.match(/\d+(\.\d+)?/g) ?? []).length >= 3)
    ) {
      const entries = [
        owner,
        ...listFunctions(ranked).filter((entry) => entry !== owner),
      ].slice(0, limits.entries);
      return finishUsage(asked, entries, confidence, {
        ...extra,
        language,
        example: findExampleEntry(entries, language),
      });
    }
    if (
      route === 'placement' &&
      target.includes('@') &&
      owner.kind === 'function'
    ) {
      const field = target.split('@')[0];
      const candidates = [
        owner,
        ...getTemplateFunctions(owner, document),
        ...document.entries,
      ];
      for (const candidate of candidates) {
        const item = findItem(candidate, field, document);
        if (item)
          return {
            ...extra,
            reason: { route: 'placement', confidence },
            placement: { entry: owner, owner: candidate, item },
            entries: [owner],
            values: null,
            link: makeLink(candidate, language),
            followUps: getUsageFollowUps(owner, null),
          };
      }
      return null;
    }
    if (route === 'definition') {
      const field = target.includes('@') ? target.split('@')[0] : null;
      const item = field ? findItem(owner, field, document) : null;
      if (field && !item) return null;
      const explanation = item
        ? {
            type: 'field',
            term: field,
            groups: [
              {
                meaning: item.meaning,
                matches: [{ entry: owner, item }],
                isHere: true,
              },
            ],
          }
        : { type: 'entry', entry: owner };
      const option =
        explanation.type === 'field' &&
        listOptionFunctions(explanation.term, document).length > 0
          ? explanation.term
          : null;
      const entries =
        explanation.type === 'entry' && owner.kind === 'function'
          ? [owner]
          : [];
      return {
        ...extra,
        reason: { route: 'definition', confidence },
        explanation,
        option,
        entries,
        values: null,
        followUps: [],
      };
    }
    return null;
  }

  function findTypedFunction(asked) {
    return (
      document.entries.find(
        (entry) => entry.kind === 'function' && isCodeForm(entry, asked),
      ) ?? null
    );
  }

  async function answerIonMz(asked, language) {
    const ion = readIonMz(asked);
    const entry = findEntry(document, 'calculate_eic');
    if (!ion || !entry) return null;
    const result = await finishUsage(asked, [entry], 'high', {
      language,
      example: findExampleEntry([entry], language),
    });
    const note = `In ${ion.mode} mode, the ${ion.name} ${ion.formula} has m/z ${ion.mass} ${ion.sign} 1.00728 = ${ion.mz}. Use ${ion.mz} as mz in calculate_eic or in your targets.`;
    return {
      ...result,
      values: { function: entry.name, mz: Number(ion.mz) },
      templateValues: {},
      faq: { note, route: 'usage', target: entry.name, score: 1 },
    };
  }

  async function answerConvert(asked, language) {
    const entry = findEntry(document, readConvertQuestion(asked) ?? '');
    if (!entry) return null;
    const result = await finishUsage(asked, [entry], 'high', {
      language,
      example: findExampleEntry([entry], language),
    });
    return { ...result, converter: true };
  }

  async function finishReview(text, language) {
    const { code } = splitProse(text);
    const review = await reviewWithFixer(
      code,
      reviewCode(code, document, language),
      language,
    );
    if (review.notes.some((note) => /^Line \d+:/.test(note))) return review;
    const error = await findSyntaxError(review.code, review.language).catch(
      () => null,
    );
    return error ? { ...review, notes: [error, ...review.notes] } : review;
  }

  async function reviewWithFixer(code, review, language) {
    if (!fixNames || review.unknown.length === 0) return review;
    const guesses = await fixNames(review.unknown);
    if (Object.keys(guesses).length === 0) return review;
    const again = reviewCode(code, document, language, guesses);
    if (again.unknown.length === 0) return again;
    return reviewCode(code, document, language, {
      ...guesses,
      ...(await fixNames(again.unknown)),
    });
  }

  function namedFunctions(question) {
    return document.entries
      .filter(
        (entry) => entry.kind === 'function' && isCodeForm(entry, question),
      )
      .map((entry) => entry.name);
  }

  function findFaqLinks(item, functions, language) {
    const known = [...item.functions, ...item.uses];
    const name =
      functions.find((candidate) => known.includes(candidate)) ??
      item.functions.find((candidate) => findEntry(document, candidate));
    const entry = name ? findEntry(document, name) : null;
    if (!entry) return [];
    const options = getOptionsType(entry, document);
    const text = [item.intro, ...item.ideas, ...Object.values(item.notes)].join(
      ' ',
    );
    const nested = (options?.fields ?? [])
      .map((field) =>
        findEntry(
          document,
          (field.languages.rust?.type ?? '').replace(/^Option<(.+)>$/, '$1'),
        ),
      )
      .filter(
        (type) =>
          type?.group === 'Options' &&
          (type.fields ?? []).some((field) =>
            text.includes(`\`${getPlainName(field)}\``),
          ),
      );
    return [entry, options, ...new Set(nested)]
      .filter(Boolean)
      .map((link) => makeLink(link, language));
  }

  function focusOf(item) {
    const named = item.functions.filter(
      (name) => findEntry(document, name)?.kind === 'function',
    );
    return named.length > 0 ? named : item.uses;
  }

  function nameFor(item, codeFunctions, language) {
    const focus = focusOf(item);
    const name =
      codeFunctions.find((candidate) => focus.includes(candidate)) ??
      codeFunctions.find((candidate) => item.uses.includes(candidate));
    const entry = name ? findEntry(document, name) : null;
    return entry ? getDisplayName(entry, language) : null;
  }

  function problemChips(ranked, codeFunctions, language, skip) {
    const ids = [
      ...ranked
        .map(({ id }) => readAnswerId(id))
        .filter(({ route }) => route === 'faq')
        .map(({ target }) => target),
      ...faqItems.map((item) => item.id),
    ];
    const chips = [];
    for (const id of new Set(ids)) {
      const item = findFaq(id);
      if (!item || id === skip || vagueFaqs.includes(id)) continue;
      if (!item.uses.some((name) => codeFunctions.includes(name))) continue;
      const chip = faqChip(item, nameFor(item, codeFunctions, language));
      if (!chips.some((other) => other.label === chip.label)) chips.push(chip);
    }
    return chips;
  }

  function pickProblem(block, codeFunctions, language, ranked = []) {
    const chips = problemChips(ranked, codeFunctions, language).slice(0, 6);
    if (chips.length === 0) return null;
    const main = [...codeFunctions].sort(
      (left, right) =>
        faqItems.filter((item) => item.uses.includes(right)).length -
        faqItems.filter((item) => item.uses.includes(left)).length,
    )[0];
    const entry = findEntry(document, main);
    return {
      reason: { route: 'faq', confidence: 'medium' },
      faq: {
        id: 'pick-problem',
        note: `Which problem do you see with \`${getDisplayName(entry, language)}\`?`,
        ideas: [],
        notes: [],
        owners: [main],
        target: main,
      },
      entries: [],
      values: null,
      link: makeLink(entry, language),
      followUps: chips.map((chip) => ({
        ...chip,
        question: `${chip.label} /code{\n${block.code}\n}`,
      })),
      followUpsLabel: 'Pick one:',
    };
  }

  function applyCode(answer, block, review, ranked, language) {
    if (answer?.reason.route !== 'faq') return;
    const item = findFaq(answer.faq.id);
    if (!item) return;
    const codeFunctions = review.entries.map((entry) => entry.name);
    const values = review.values.filter(
      (value, index, all) =>
        all.findIndex((other) => other.name === value.name) === index,
    );
    const fill = (text) => fillValues(text, values, item);
    const firedRules = item.when.filter((rule) =>
      matchesWhen(rule, values, item),
    );
    const fired = firedRules.map((rule) => rule.text);
    const ideas = [
      ...item.ideas.filter(
        (idea) =>
          !firedRules.some((rule) =>
            rule.terms.every((term) =>
              mentionedNames(idea).includes(term.name),
            ),
          ),
      ),
      ...fired,
    ];
    answer.faq.note = fill(item.intro);
    answer.faq.ideas = [
      ...ideas.filter((idea) => mentionsSet(idea, values)),
      ...ideas.filter((idea) => !mentionsSet(idea, values)),
    ].map(fill);
    answer.faq.notes = pickFaqNotes(item, block.question, codeFunctions).map(
      fill,
    );
    answer.links = findFaqLinks(
      item,
      [...codeFunctions, ...namedFunctions(block.question ?? '')],
      language,
    );
    answer.link = answer.links[0] ?? answer.link;
    answer.faq.fromCode = values.filter((value) =>
      item.options.some((option) => option.name === value.name),
    );
    const chips = problemChips(ranked, codeFunctions, language, item.id)
      .slice(0, 3)
      .map((chip) => ({
        ...chip,
        question: `${chip.label} /code{\n${block.code}\n}`,
      }));
    if (chips.length > 0) {
      answer.followUps = [
        ...chips,
        {
          label: 'Not this one',
          question: `${block.question} /code{\n${block.code}\n}`,
          faqId: 'pick-problem',
        },
      ];
      answer.followUpsLabel = 'Or is it:';
    }
  }

  async function answerQuestion(question, language, code = null) {
    const raw = question.slice(
      0,
      isCommand(question) || looksLikeCode(question)
        ? limits.code
        : limits.question,
    );
    if (!code && isCommand(raw)) {
      let command = runCommand(raw, document, language);
      if (command.command === 'check' && !command.open) {
        const review = await finishReview(
          raw.replace(/^\s*\/check\s*/, ''),
          language,
        );
        command =
          review.entries.length > 0 ||
          review.notes.length > 0 ||
          review.fixes.length > 0
            ? { command: 'check', review }
            : {
                command: 'check',
                failure: 'I found no Quant·ion function in that code.',
              };
      }
      return {
        reason: { route: 'command', confidence: 'high' },
        command,
        entries: command.entry
          ? [command.entry]
          : (command.review?.entries ?? []),
        values: null,
        followUps: [],
      };
    }
    if (!code && looksLikeCode(raw)) {
      const review = await finishReview(raw, language);
      if (review.entries.length > 0 || review.fixes.length > 0)
        return {
          reason: { route: 'review', confidence: 'high' },
          review,
          entries: review.entries,
          values: null,
          link: makeLink(review.entries[0], language),
          followUps: [],
        };
    }
    const ionAnswer = await answerIonMz(raw, language);
    if (ionAnswer) return ionAnswer;
    const convertAnswer = await answerConvert(raw, language);
    if (convertAnswer) return convertAnswer;
    if (!encode || !classify) return { ...refusedResult(), noModel: true };
    const questionVector = await encode(raw);
    let ranked = classify(questionVector);
    const codeFunctions = code
      ? code.review.entries.map((entry) => entry.name)
      : [];
    const top = readAnswerId(ranked[0].id);
    const confident =
      top.route !== 'faq' &&
      ranked[0].probability >= limits.keepOverCode &&
      (codeFunctions.length === 0 ||
        codeFunctions.includes(findFunction(top.target)?.name));
    if (code && !confident)
      ranked = rerankFaqs(
        ranked,
        focusOf,
        codeFunctions,
        code.review.values,
        scoreCode?.(
          questionVector,
          codeFunctions,
          code.review.values.map((value) => value.name),
        ) ?? null,
      );
    const typed = findTypedFunction(raw);
    if (typed && ranked[0].id === 'refuse') {
      const about = ranked.find(
        ({ id }) =>
          id !== 'refuse' &&
          id !== 'unsupported' &&
          findFunction(readAnswerId(id).target) === typed,
      );
      ranked = about
        ? [about, ...ranked.filter((item) => item !== about)]
        : [{ id: `usage:${typed.name}`, probability: 1 }, ...ranked];
    }
    for (const { id, probability, confidence } of ranked.slice(
      0,
      limits.entries,
    )) {
      const result = await answerFromId(
        id,
        probability,
        ranked,
        raw,
        questionVector,
        language,
        Boolean(code),
      );
      if (
        result?.reason.route === 'usage' &&
        ['write_ion', 'write_mzml'].includes(result.entries[0]?.name)
      )
        return { ...result, converter: true };
      if (result?.reason.route === 'faq') {
        const sure = code ? confidence : probability;
        if (sure < (code ? limits.faqSureWithCode : limits.lowProbability))
          return { ...notSureResult({ questionVector }), ranked };
      }
      if (result) return { ...result, ranked };
    }
    return refusedResult({ questionVector });
  }

  const asker = async (question, language, options) => {
    const faqId = options?.faqId;
    const text = question.slice(0, limits.code);
    const split =
      !isCommand(text) && looksLikeCode(text) ? splitProse(text) : null;
    const block =
      readCodeBlock(text) ??
      (split?.prose ? { question: split.prose, code: split.code } : null);
    if (block) {
      const review = await finishReview(block.code, language);
      const codeFunctions = review.entries.map((entry) => entry.name);
      const answer =
        faqId && faqId !== 'pick-problem'
          ? await answerFromId(
              `faq:${faqId}`,
              1,
              [],
              block.question,
              null,
              language,
            )
          : block.question
            ? await answerQuestion(block.question, language, { review })
            : null;
      const ranked = answer?.ranked ?? [];
      applyCode(answer, block, review, ranked, language);
      const asksForCode =
        faqId === 'pick-problem' ||
        ['error-unclear', 'vague-wrong'].includes(answer?.faq?.id);
      const isProblem =
        asksForCode ||
        (!faqId &&
          !['faq', 'not-sure'].includes(answer?.reason.route) &&
          problemPattern.test(block.question ?? ''));
      const picked = isProblem
        ? pickProblem(block, codeFunctions, language, ranked)
        : null;
      return {
        reason: { route: 'check', confidence: 'high' },
        review,
        answer: picked ?? (asksForCode ? null : answer),
        questionVector: answer?.questionVector,
        entries: review.entries,
        values: null,
        link: makeLink(review.entries[0], language),
        followUps: [],
      };
    }
    return answerQuestion(question, language);
  };

  asker.reread = async (question, result, entryName, language) => {
    const entry = findEntry(document, entryName);
    if (!entry || entry.kind !== 'function') return result;
    const asked = question.slice(0, limits.question);
    const entries = [
      entry,
      ...result.entries.filter((item) => item !== entry),
    ].slice(0, limits.entries);
    return finishUsage(asked, entries, 'high', {
      questionVector: result.questionVector,
      language,
      example: findExampleEntry(entries, language),
    });
  };

  return asker;
}
