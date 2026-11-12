import { createContext, useContext, useEffect, useRef, useState } from 'react';

import {
  createAsker,
  asksForCode,
  createClassifier,
  createCodeScorer,
  createEncoder,
  createNameFixer,
  getStarters,
  listUnderstood,
  writeAnswer,
  writePlacement,
} from '../ask/ask';
import { commandList } from '../ask/commands';
import { HelperNotice } from './LlmLink';
import {
  convertFile,
  convertLimits,
  isTooBig,
  readFileFormat,
} from '../ask/convert';
import { writeExplanation } from '../ask/meaning';
import { issuesLink, limits, models, refusal } from '../ask/models';
import phrases from '../ask/phrases.json';
import {
  detectLanguage,
  languageLabels,
  looksLikeCode,
  splitProse,
} from '../ask/review';
import { readCodeBlock } from '../ask/code';
import { splitIntoTokens } from '../highlight';
import { runtimeFiles } from '../ask/runtime';
import { buildTemplateSnippet, getOptionsType } from '../ask/snippet';
import { getDisplayName } from '../names';
import {
  actions,
  findApiFile,
  findEntry,
  findLanguage,
} from '../state/documentationState';
import { useDocumentation } from '../state/DocumentationContext';
import { CodeBlock } from './CodeBlock';

let encoderPromise = null;
let classifierPromise = null;
const loaded = new Map();
const modelFailure =
  'The search model could not load, so I can only run commands, definitions and code checks for now.';

function getDevice() {
  if (typeof navigator !== 'undefined' && navigator.gpu) return 'webgpu';
  return 'wasm';
}

function trackProgress(onProgress) {
  return (event) => {
    if (event.status !== 'progress' || !event.total) return;
    loaded.set(event.file, { loaded: event.loaded, total: event.total });
    let done = 0;
    let total = 0;
    for (const item of loaded.values()) {
      done += item.loaded;
      total += item.total;
    }
    onProgress(Math.round((100 * done) / total));
  };
}

function loadEncoder(onProgress) {
  if (!encoderPromise) {
    const base = import.meta.env.BASE_URL;
    encoderPromise = createEncoder({
      device: getDevice(),
      onProgress: trackProgress(onProgress),
      modelFolder: `${base}${models.folder}`,
      runtimeFiles,
    });
    encoderPromise.catch(() => {
      encoderPromise = null;
    });
  }
  return encoderPromise;
}

async function readClassifier() {
  const folder = `${import.meta.env.BASE_URL}${models.folder}${models.encoder.name}/`;
  const [labels, weights] = await Promise.all([
    fetch(`${folder}${models.answers.labels}`),
    fetch(`${folder}${models.answers.weights}`),
  ]);
  if (!labels.ok || !weights.ok) throw new Error('No Ask answer list');
  return createClassifier(await labels.json(), await weights.arrayBuffer());
}

const layerPromises = {};

async function readLayer(meta) {
  const folder = `${import.meta.env.BASE_URL}${models.folder}${models.encoder.name}/`;
  const [labels, weights] = await Promise.all([
    fetch(`${folder}${meta.labels}`),
    fetch(`${folder}${meta.weights}`),
  ]);
  if (!labels.ok || !weights.ok) throw new Error(`No ${meta.labels} layer`);
  return { meta: await labels.json(), buffer: await weights.arrayBuffer() };
}

function loadLayer(meta) {
  if (!layerPromises[meta.labels]) {
    layerPromises[meta.labels] = readLayer(meta);
    layerPromises[meta.labels].catch(() => {
      delete layerPromises[meta.labels];
    });
  }
  return layerPromises[meta.labels];
}

function loadClassifier() {
  if (!classifierPromise) {
    classifierPromise = readClassifier();
    classifierPromise.catch(() => {
      classifierPromise = null;
    });
  }
  return classifierPromise;
}

function hasItem(entry, name) {
  return [...(entry.arguments ?? []), ...(entry.fields ?? [])].some((item) =>
    Object.values(item.languages ?? {}).some(
      (language) => language?.name === name,
    ),
  );
}

function findLinkTarget(name, document, owners) {
  const bare = name.replace(/\(\)$/, '');
  const entry = document.entries.find(
    (item) => item.name === bare || getDisplayName(item, 'js') === bare,
  );
  if (entry) return entry.name;
  const group = document.groups.find(
    (item) => item.name.toLowerCase() === bare.toLowerCase(),
  );
  if (group) return group.name;
  for (const owner of owners) {
    const ownerEntry = findEntry(document, owner);
    if (!ownerEntry) continue;
    if (hasItem(ownerEntry, bare)) return ownerEntry.name;
    const options = getOptionsType(ownerEntry, document);
    if (options && hasItem(options, bare)) return options.name;
  }
  return document.entries.find((item) => hasItem(item, bare))?.name ?? null;
}

function AnswerText({ text, owners = [] }) {
  const { state, dispatch } = useDocumentation();
  return text
    .split(/(`[^`]+`|https?:\/\/[^\s`]+[^\s`.,)])/)
    .map((part, position) => {
      if (/^https?:\/\//.test(part)) {
        return (
          <a
            key={position}
            className="text-link"
            href={part}
            target="_blank"
            rel="noreferrer"
          >
            {part}
          </a>
        );
      }
      if (!part.startsWith('`')) return part;
      const name = part.slice(1, -1);
      const target = findLinkTarget(
        name,
        state.document,
        owners.filter(Boolean),
      );
      if (!target) return <code key={position}>{name}</code>;
      return (
        <button
          key={position}
          type="button"
          className="ask-code-link"
          onClick={() =>
            dispatch({ type: actions.entryChosen, entryName: target })
          }
        >
          <code>{name}</code>
        </button>
      );
    });
}

function DocsLink({ link }) {
  const { dispatch } = useDocumentation();
  return (
    <button
      type="button"
      className="text-link"
      onClick={() =>
        dispatch({ type: actions.entryChosen, entryName: link.entryName })
      }
    >
      {link.label}
    </button>
  );
}

function Source({ link, links }) {
  const shown = links?.length ? links : link ? [link] : [];
  if (shown.length === 0) return null;
  return (
    <p className="ask-sources">
      For more information, see{' '}
      {shown.map((item, index) => (
        <span key={item.entryName}>
          {index > 0 && ' and '}
          <DocsLink link={item} />
        </span>
      ))}
      .
    </p>
  );
}

function Chips({ items, label, onPick, className = '' }) {
  if (!items || items.length === 0) return null;
  return (
    <div className={`ask-chips ${className}`}>
      {label && <span className="ask-chips-label">{label}</span>}
      {items.map((item, position) => (
        <button
          key={`${item.label}-${position}`}
          type="button"
          className="ask-chip"
          onClick={() => onPick(item)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}

function QuestionChips({ items, label }) {
  const { askQuestion } = useContext(AskContext);
  return (
    <Chips
      items={items}
      label={label}
      onPick={(item) =>
        askQuestion(item.question ?? item.label, { faqId: item.faqId })
      }
    />
  );
}

function ExplanationResult({ explanation }) {
  const { state } = useDocumentation();
  const { main, link } = writeExplanation(
    explanation,
    state.document,
    state.language,
  );
  return (
    <div className="ask-reply">
      <p className="ask-answer">
        <AnswerText text={main} owners={[link?.entryName]} />
      </p>
      <Source link={link} />
    </div>
  );
}

function ReviewResult({ review, link, showsCode = true }) {
  const { state } = useDocumentation();
  const problems = review.fixes.length + review.notes.length;
  return (
    <div className="ask-reply">
      <p className="ask-answer">
        {problems === 0
          ? `No problems found: every Quant·ion function and option in this ${languageLabels[review.language]} code exists in the API.`
          : `Found ${problems} problem${problems > 1 ? 's' : ''} in this ${languageLabels[review.language]} code.`}
      </p>
      {problems > 0 && (
        <ul className="ask-meanings">
          {review.fixes.map((fix) => (
            <li key={`${fix.from}-${fix.to}`}>
              <AnswerText
                text={`${fix.line ? `Line ${fix.line}: ` : ''}\`${fix.from}\` → \`${fix.to}\``}
                owners={review.entries.map((item) => item.name)}
              />
              <span className="ask-fix-reason">
                <AnswerText
                  text={fix.reason}
                  owners={review.entries.map((item) => item.name)}
                />
              </span>
            </li>
          ))}
          {review.notes.map((note) => (
            <li key={note}>
              <AnswerText
                text={note}
                owners={review.entries.map((item) => item.name)}
              />
            </li>
          ))}
        </ul>
      )}
      {showsCode && review.fixes.length > 0 && (
        <CodeBlock
          title={`Corrected ${languageLabels[review.language]} code`}
          code={review.code}
          language={review.language}
          numbered
        />
      )}
      {state.language !== review.language && (
        <p className="ask-status">
          Checked as {languageLabels[review.language]}, from the code's syntax.
        </p>
      )}
      <Source
        link={
          link ??
          (review.entries[0] && {
            entryName: review.entries[0].name,
            label: getDisplayName(review.entries[0], review.language),
          })
        }
      />
    </div>
  );
}

function pickPhrase(list, text) {
  return list[
    [...text].reduce((sum, character) => sum + character.charCodeAt(0), 0) %
      list.length
  ];
}

function Unsure({ message, result }) {
  const { state } = useDocumentation();
  const alternatives = (result.entries ?? []).map((entry) => ({
    label: getDisplayName(entry, state.language),
    question: `How do I use ${entry.name}?`,
  }));
  return (
    <div className="ask-reply">
      <p className="ask-answer">
        {pickPhrase(phrases.unsure, message.question)}
      </p>
      <QuestionChips items={alternatives} />
    </div>
  );
}

function Refusal({ result }) {
  const { state } = useDocumentation();
  const alternatives = (result.entries ?? []).map((entry) => ({
    label: getDisplayName(entry, state.language),
    question: `How do I use ${entry.name}?`,
  }));
  return (
    <div className="ask-reply">
      <p className="ask-answer">{refusal}</p>
      {result.noModel && <p className="ask-status">{modelFailure}</p>}
      <QuestionChips items={alternatives} label="Did you mean:" />
      <QuestionChips items={getStarters()} label="Try one of these:" />
    </div>
  );
}

function InstallList({ languages }) {
  const { state } = useDocumentation();
  const shown = languages.filter((item) => item.key === state.language);
  return (
    <ul className="ask-install">
      {shown.map((item) => (
        <li key={item.key}>
          <strong>{item.label}</strong>
          {item.registry ? (
            <p>
              <a
                className="text-link"
                href={item.registry.address}
                target="_blank"
                rel="noreferrer"
              >
                {item.registry.label}
              </a>
              : <code>{item.command}</code>
            </p>
          ) : (
            <p>{item.pending}</p>
          )}
          {item.github && (
            <p>
              <a
                className="text-link"
                href={item.github.address}
                target="_blank"
                rel="noreferrer"
              >
                GitHub
              </a>
              : <code>{item.github.command}</code>
            </p>
          )}
          {item.platforms && (
            <p className="ask-install-note">{item.platforms}</p>
          )}
        </li>
      ))}
    </ul>
  );
}

function GuideResult({ result }) {
  const { state } = useDocumentation();
  const answer = result.answer ?? result.guide.answer;
  const functions = result.entries
    .filter((entry) => !answer.includes(`\`${entry.name}\``))
    .map((entry) => ({
      entryName: entry.name,
      label: getDisplayName(entry, state.language),
    }));
  return (
    <div className="ask-reply">
      <p className="ask-answer">
        <AnswerText text={answer} owners={result.guide.functions} />
      </p>
      {result.fileNote && (
        <p className="ask-answer">
          <AnswerText
            text={result.fileNote}
            owners={['parse_ion', 'parse_mzml']}
          />
        </p>
      )}
      {functions.length > 0 && (
        <p className="ask-sources">
          Functions:{' '}
          {functions.map((link) => (
            <DocsLink key={link.entryName} link={link} />
          ))}
        </p>
      )}
      {result.guide.languages && (
        <InstallList languages={result.guide.languages} />
      )}
      {result.guide.links && (
        <p className="ask-sources">
          {result.guide.links.map((link) => (
            <a
              key={link.address}
              className="text-link"
              href={link.address}
              target="_blank"
              rel="noreferrer"
            >
              {link.label} ↗
            </a>
          ))}
        </p>
      )}
      <Source link={result.link} />
      <QuestionChips items={result.followUps} />
    </div>
  );
}

function FaqResult({ result }) {
  const fromCode = (result.faq.fromCode ?? [])
    .map((value) => `\`${value.name}\` = ${value.value}`)
    .join(', ');
  return (
    <div className="ask-reply">
      {fromCode && (
        <p className="ask-from-code">
          <AnswerText
            text={`From your code: ${fromCode}.`}
            owners={result.faq.owners}
          />
        </p>
      )}
      <p className="ask-answer">
        <AnswerText
          text={result.faq.note}
          owners={result.faq.owners ?? [result.faq.target.split('@').pop()]}
        />
      </p>
      {[...(result.faq.ideas ?? []), ...(result.faq.notes ?? [])].length >
        0 && (
        <ul className="ask-meanings ask-faq-ideas">
          {[...(result.faq.ideas ?? []), ...(result.faq.notes ?? [])].map(
            (idea) => (
              <li key={idea}>
                <AnswerText text={idea} owners={result.faq.owners} />
              </li>
            ),
          )}
        </ul>
      )}
      <Source link={result.link} links={result.links} />
      <QuestionChips
        items={result.followUps}
        label={result.followUpsLabel ?? 'Related:'}
      />
    </div>
  );
}

function ListResult({ result }) {
  const { state } = useDocumentation();
  const { option, entries } = result.list;
  const names = entries.map(
    (entry) => `\`${getDisplayName(entry, state.language)}\``,
  );
  const text =
    entries.length === 1
      ? `Only ${names[0]} takes \`${option}\`.`
      : `These functions take \`${option}\`: ${names.join(', ')}.`;
  return (
    <div className="ask-reply">
      <p className="ask-answer">
        <AnswerText text={text} owners={entries.map((entry) => entry.name)} />
      </p>
      <QuestionChips items={result.followUps} label="Show me:" />
    </div>
  );
}

function readDefault(document, ownerName, name, fallback) {
  const owner = findEntry(document, ownerName);
  const item = owner?.fields?.find(
    (field) => field.languages.python?.name === name,
  );
  const value = Number(item?.languages.js?.default);
  return Number.isFinite(value) ? value : fallback;
}

function formatSize(bytes) {
  if (bytes >= 1000 * 1000) return `${(bytes / 1000 / 1000).toFixed(1)} MB`;
  return `${Math.max(1, Math.round(bytes / 1000))} kB`;
}

function ConvertTool() {
  const { state } = useDocumentation();
  const [file, setFile] = useState(null);
  const [isOver, setIsOver] = useState(false);
  const [mzWindow, setMzWindow] = useState(() =>
    readDefault(state.document, 'WriteIonOptions', 'mz_window', 20),
  );
  const [blockSize, setBlockSize] = useState(() =>
    readDefault(state.document, 'WriteIonOptions', 'block_size', 8),
  );
  const [status, setStatus] = useState({ kind: 'idle' });
  const format = file ? readFileFormat(file.name) : null;
  const tooBig = Boolean(format) && isTooBig(file, format);
  const isBusy = status.kind === 'busy';
  const usesOptions = !format || format.target === 'ion';
  const problem =
    file && !format
      ? 'Only .ion and .mzML files can be converted.'
      : tooBig
        ? `${format.label} files over ${format.megabytes} MB cannot be converted here. Convert it with the library instead.`
        : null;

  useEffect(
    () => () => status.address && URL.revokeObjectURL(status.address),
    [status.address],
  );

  function choose(chosen) {
    if (!chosen || isBusy) return;
    setFile(chosen);
    setStatus({ kind: 'idle' });
  }

  function drop(event) {
    event.preventDefault();
    setIsOver(false);
    choose(event.dataTransfer.files?.[0]);
  }

  async function convert(event) {
    event.preventDefault();
    if (!format || tooBig) return;
    if (
      format.target === 'ion' &&
      (!Number.isInteger(mzWindow) ||
        mzWindow < 0 ||
        !Number.isInteger(blockSize) ||
        blockSize < 1)
    ) {
      setStatus({
        kind: 'failed',
        message:
          'The m/z window must be a whole number, 0 or more, and the block size a whole number, 1 or more.',
      });
      return;
    }
    setStatus({ kind: 'busy' });
    try {
      const converted = await convertFile(
        file,
        format.target === 'ion' ? { mzWindow, blockSize } : {},
      );
      const address = URL.createObjectURL(converted.blob);
      setStatus({
        kind: 'done',
        address,
        name: converted.name,
        size: converted.blob.size,
      });
      const link = document.createElement('a');
      link.href = address;
      link.download = converted.name;
      link.click();
    } catch (error) {
      setStatus({ kind: 'failed', message: error.message });
    }
  }

  return (
    <form className="ask-convert" onSubmit={convert}>
      <label
        className={`ask-convert-drop${isOver ? ' ask-convert-over' : ''}${isBusy ? ' ask-convert-disabled' : ''}`}
        onDragOver={(event) => {
          event.preventDefault();
          setIsOver(true);
        }}
        onDragLeave={() => setIsOver(false)}
        onDrop={drop}
      >
        <input
          type="file"
          className="ask-convert-file"
          accept=".ion,.mzML,.mzml"
          onChange={(event) => choose(event.target.files?.[0])}
          disabled={isBusy}
        />
        <span className="ask-convert-pick">
          {file ? 'Choose another file' : 'Choose a file'}
        </span>
        <span className="ask-convert-name">
          {file
            ? `${file.name} · ${formatSize(file.size)}`
            : `or drop a file here: .ion up to ${convertLimits.ion} MB, .mzML up to ${convertLimits.mzml} MB`}
        </span>
      </label>
      <fieldset
        className="ask-convert-options"
        disabled={isBusy || !usesOptions}
      >
        <legend>
          {usesOptions
            ? 'Options for mzML to .ion'
            : 'No options for .ion to mzML'}
        </legend>
        <label>
          <span>m/z window</span>
          <input
            type="number"
            min="0"
            step="1"
            value={mzWindow}
            onChange={(event) => setMzWindow(Number(event.target.value))}
          />
          <span className="ask-convert-unit">Da</span>
        </label>
        <label>
          <span>Block size</span>
          <input
            type="number"
            min="1"
            step="1"
            value={blockSize}
            onChange={(event) => setBlockSize(Number(event.target.value))}
          />
          <span className="ask-convert-unit">MB</span>
        </label>
      </fieldset>
      <p className="ask-convert-warning">
        Large files use a lot of memory and can crash the page. For big files,
        use the library.
      </p>
      {problem && <p className="ask-status ask-rejected">{problem}</p>}
      <div className="ask-convert-footer">
        <button
          type="submit"
          className="ask-button"
          disabled={isBusy || !format || tooBig}
        >
          {isBusy
            ? 'Converting…'
            : format
              ? `Convert to ${format.target === 'ion' ? '.ion' : '.mzML'}`
              : 'Convert'}
        </button>
        {status.kind === 'done' && (
          <span className="ask-status">
            Done:{' '}
            <a
              className="text-link"
              href={status.address}
              download={status.name}
            >
              {status.name}
            </a>{' '}
            · {formatSize(status.size)}
          </span>
        )}
        {status.kind === 'failed' && (
          <span className="ask-status ask-rejected">{status.message}</span>
        )}
      </div>
    </form>
  );
}

function ConvertOffer() {
  return (
    <>
      <p className="ask-answer">
        Or convert it here, with nothing to install. The file stays on your
        computer.
      </p>
      <ConvertTool />
    </>
  );
}

function Tokens({ code, language }) {
  return splitIntoTokens(code, language).map((token, index) =>
    token.kind ? (
      <span key={index} className={`token-${token.kind}`}>
        {token.text}
      </span>
    ) : (
      token.text
    ),
  );
}

function LineNumbers({ code, innerRef, className = '' }) {
  return (
    <pre
      ref={innerRef}
      className={`ask-line-numbers ${className}`}
      aria-hidden="true"
    >
      {code
        .split('\n')
        .map((_, index) => index + 1)
        .join('\n')}
    </pre>
  );
}

function CheckWindow() {
  const { state } = useDocumentation();
  const { askQuestion, isBusy } = useContext(AskContext);
  const [code, setCode] = useState('');
  const [isSent, setIsSent] = useState(false);
  const shownRef = useRef(null);
  const numbersRef = useRef(null);

  function check(event) {
    event.preventDefault();
    if (!code.trim() || isBusy) return;
    setIsSent(true);
    askQuestion(`/check\n${code.trim()}`);
  }

  if (isSent) return null;
  return (
    <form className="ask-reply ask-check" onSubmit={check}>
      <p className="ask-answer">
        Paste only the Quant·ion calls and their arguments, not your whole
        script.
      </p>
      <div className="ask-check-editor">
        <LineNumbers
          code={code}
          innerRef={numbersRef}
          className="ask-check-numbers"
        />
        <pre ref={shownRef} className="ask-check-shown" aria-hidden="true">
          <Tokens
            code={`${code}\n`}
            language={detectLanguage(code, state.language)}
          />
        </pre>
        <textarea
          className="ask-check-code"
          value={code}
          maxLength={limits.code}
          autoFocus
          spellCheck={false}
          placeholder={'peaks = find_peaks(eic["x"], eic["y"], {"min_snr": 3})'}
          onChange={(event) => setCode(event.target.value)}
          onScroll={(event) => {
            if (numbersRef.current)
              numbersRef.current.scrollTop = event.target.scrollTop;
            if (!shownRef.current) return;
            shownRef.current.scrollTop = event.target.scrollTop;
            shownRef.current.scrollLeft = event.target.scrollLeft;
          }}
          onKeyDown={(event) => {
            if (
              event.key !== 'Enter' ||
              event.shiftKey ||
              event.nativeEvent.isComposing
            )
              return;
            check(event);
          }}
        />
      </div>
      <div className="ask-check-foot">
        <span className="ask-count">
          {code.length}/{limits.code}
        </span>
        <button
          type="submit"
          className="ask-button"
          disabled={!code.trim() || isBusy}
        >
          Check
        </button>
      </div>
    </form>
  );
}

function CommandResult({ result }) {
  const { askQuestion } = useContext(AskContext);
  const { command } = result;
  if (command.command === 'help' || command.command === 'unknown') {
    return (
      <div className="ask-reply">
        {command.failure && <p className="ask-answer">{command.failure}</p>}
        <ul className="ask-meanings">
          {command.commands.map((item) => (
            <li key={item.name}>
              <code>{item.usage}</code> {item.meaning}
            </li>
          ))}
        </ul>
        {command.examples && (
          <Chips
            items={command.examples.map((question) => ({ label: question }))}
            label="Try:"
            onPick={(item) => askQuestion(item.label)}
          />
        )}
      </div>
    );
  }
  if (command.command === 'convert') {
    return (
      <div className="ask-reply">
        <p className="ask-answer">
          Pick a .ion file to get mzML, or a .mzML file to get .ion. The file
          stays on your computer.
        </p>
        <ConvertTool />
      </div>
    );
  }
  if (command.command === 'check') {
    if (command.open) return <CheckWindow />;
    if (command.failure) {
      return (
        <div className="ask-reply">
          <p className="ask-answer">
            <AnswerText text={command.failure} />
          </p>
        </div>
      );
    }
    return <ReviewResult review={command.review} />;
  }
  return null;
}

function ValueChip({ chip, onChange }) {
  const [isEditing, setIsEditing] = useState(false);
  const [text, setText] = useState(String(chip.value));
  if (!isEditing) {
    return (
      <button
        type="button"
        className="ask-chip ask-chip-understood"
        title="Change this value"
        onClick={() => setIsEditing(true)}
      >
        {chip.label}
      </button>
    );
  }
  function finish() {
    setIsEditing(false);
    const value =
      typeof chip.value === 'number'
        ? Number(text)
        : typeof chip.value === 'boolean'
          ? text.trim().toLowerCase() === 'true'
          : text.trim();
    if (typeof chip.value === 'number' && !Number.isFinite(value)) return;
    onChange(value);
  }
  return (
    <form
      className="ask-chip ask-chip-edit"
      onSubmit={(event) => {
        event.preventDefault();
        finish();
      }}
    >
      <span>{chip.name} =</span>
      <input
        type="text"
        value={text}
        autoFocus
        aria-label={`New value for ${chip.name}`}
        onChange={(event) => setText(event.target.value)}
        onBlur={finish}
      />
    </form>
  );
}

function CodeSteps({ result }) {
  const { state } = useDocumentation();
  const steps = listUnderstood(result, state.document, state.language)
    .filter((chip) => chip.kind === 'value')
    .map((chip) => {
      const owner = findEntry(state.document, chip.owner);
      const name = chip.label.split(' = ')[0];
      return `Set \`${name}\` to ${chip.label.slice(name.length + 3).replace(/ \(.*\)$/, '')} in \`${owner ? getDisplayName(owner, state.language) : chip.owner}\`.`;
    });
  if (steps.length < 2) return null;
  return (
    <ul className="ask-meanings ask-faq-ideas">
      {steps.map((step) => (
        <li key={step}>
          <AnswerText
            text={step}
            owners={result.entries.map((item) => item.name)}
          />
        </li>
      ))}
    </ul>
  );
}

function Understood({ message }) {
  const { state } = useDocumentation();
  const { chooseFunction, changeValue } = useContext(AskContext);
  const [isPicking, setIsPicking] = useState(false);
  const chips = listUnderstood(message.result, state.document, state.language);
  const alternatives = message.result.entries.slice(1);
  return (
    <div className="ask-chips ask-understood">
      <span className="ask-chips-label">I read:</span>
      {chips.map((chip) =>
        chip.kind === 'function' ? (
          <span key="function" className="ask-pick">
            <button
              type="button"
              className="ask-chip ask-chip-understood"
              title="Change the function"
              onClick={() => setIsPicking((current) => !current)}
            >
              {chip.label}
            </button>
            {isPicking && alternatives.length > 0 && (
              <span className="ask-pick-menu">
                {alternatives.map((entry) => (
                  <button
                    key={entry.name}
                    type="button"
                    className="ask-chip"
                    onClick={() => {
                      setIsPicking(false);
                      chooseFunction(message.id, entry.name);
                    }}
                  >
                    {getDisplayName(entry, state.language)}
                  </button>
                ))}
              </span>
            )}
          </span>
        ) : (
          <ValueChip
            key={`${chip.owner}.${chip.name}`}
            chip={chip}
            onChange={(value) =>
              changeValue(message.id, chip.owner, chip.name, value)
            }
          />
        ),
      )}
    </div>
  );
}

function UsageResult({ message }) {
  const { state } = useDocumentation();
  const { chooseFunction } = useContext(AskContext);
  const { result } = message;
  const { language } = state;
  const { confidence } = result.reason;
  if (confidence === 'low')
    return result.entries?.length ? (
      <Unsure message={message} result={result} />
    ) : (
      <Refusal result={result} />
    );
  const languageLabel =
    findLanguage(state.document, language)?.label ?? language;
  const entry = result.entries[0];
  const snippet = buildTemplateSnippet(
    result.values,
    result.templateValues,
    state.document,
    language,
  );
  const variantCode =
    result.variant && !result.variant.refused ? result.variant[language] : null;
  const hasOwnValues =
    Object.keys(result.values ?? {}).length > 1 ||
    Object.keys(result.templateValues ?? {}).length > 0;
  let code = null;
  let codeEntry = entry;
  if (variantCode && !hasOwnValues) code = variantCode;
  else if (snippet.valid) {
    code = snippet.code;
    codeEntry = findEntry(state.document, snippet.function);
  } else if (result.example) {
    code = result.example.languages[language].example;
    codeEntry = result.example;
  }
  return (
    <div className="ask-reply">
      <p className="ask-answer">
        {pickPhrase(phrases.sure, message.question)}{' '}
        <AnswerText
          text={
            result.faq
              ? result.faq.note
              : writeAnswer(result, state.document, language)
          }
          owners={result.entries.map((item) => item.name)}
        />
      </p>
      {result.fileNote && (
        <p className="ask-answer">
          <AnswerText
            text={result.fileNote}
            owners={['parse_ion', 'parse_mzml']}
          />
        </p>
      )}
      {confidence === 'medium' && (
        <Chips
          items={result.alternatives}
          label="Did you mean:"
          onPick={(item) => chooseFunction(message.id, item.entryName)}
        />
      )}
      {message.fromCode ? (
        <CodeSteps result={result} />
      ) : (
        <Understood message={message} />
      )}
      {code && !message.fromCode && asksForCode(message.question) && (
        <CodeBlock
          title={`Template from the docs · ${languageLabel} · ${getDisplayName(codeEntry, language)}`}
          code={code}
          language={language}
        />
      )}
      {result.converter && <ConvertOffer />}
      <Source link={result.link} />
      <QuestionChips items={result.followUps} />
    </div>
  );
}

function AskResult({ message }) {
  const { state } = useDocumentation();
  const { language } = state;
  const { result } = message;
  const { route } = result.reason;
  if (route === 'check')
    return (
      <>
        {result.answer && (
          <AskResult
            message={{ ...message, result: result.answer, fromCode: true }}
          />
        )}
        {(!result.answer ||
          result.review.fixes.length + result.review.notes.length > 0) && (
          <ReviewResult
            review={result.review}
            link={result.answer ? null : result.link}
            showsCode={false}
          />
        )}
      </>
    );
  if (route === 'not-sure')
    return (
      <div className="ask-reply">
        <p className="ask-answer">
          I'm not sure. Please report the issue in{' '}
          <a href={issuesLink} target="_blank" rel="noreferrer">
            {issuesLink}
          </a>
          .
        </p>
      </div>
    );
  if (route === 'refused') return <Refusal result={result} />;
  if (route === 'command') return <CommandResult result={result} />;
  if (route === 'list') return <ListResult result={result} />;
  if (route === 'faq' || route === 'unsupported')
    return <FaqResult result={result} />;
  if (route === 'review')
    return <ReviewResult review={result.review} link={result.link} />;
  if (route === 'definition')
    return <ExplanationResult explanation={result.explanation} />;
  if (route === 'guide') return <GuideResult result={result} />;
  if (route === 'placement') {
    const placement = writePlacement(result.placement, language);
    const languageLabel =
      findLanguage(state.document, language)?.label ?? language;
    return (
      <div className="ask-reply">
        <p className="ask-answer">
          <AnswerText
            text={placement.text}
            owners={[placement.entry?.name, result.link?.entryName]}
          />
        </p>
        {placement.line &&
          !message.fromCode &&
          asksForCode(message.question) && (
            <CodeBlock
              title={`Template from the docs · ${languageLabel} · ${getDisplayName(placement.entry, language)}`}
              code={placement.line}
              language={language}
            />
          )}
        <Source link={result.link} />
        <QuestionChips items={result.followUps} />
      </div>
    );
  }
  return <UsageResult message={message} />;
}

function CodeExample() {
  const { state } = useDocumentation();
  const { askQuestion } = useContext(AskContext);
  const entry = findEntry(state.document, 'find_peaks');
  const code = entry?.languages[state.language]?.example ?? '';
  if (!entry || !code) return null;
  const question = `${getDisplayName(entry, state.language)} is not returning peaks`;
  return (
    <button
      type="button"
      className="ask-question ask-question-code ask-code-example"
      title="Try this example"
      onClick={() => askQuestion(`${question} /code{\n${code}\n}`)}
    >
      <span className="ask-question-prose">
        {question} <code className="ask-question-command">/code{'{'}</code>
      </span>
      <span className="ask-question-lines">
        <pre>
          <code>
            <Tokens code={code} language={state.language} />
          </code>
        </pre>
      </span>
      <code className="ask-question-command">{'}'}</code>
    </button>
  );
}

function Question({ text }) {
  const { state } = useDocumentation();
  const block = readCodeBlock(text);
  const fix = Boolean(block) || /^\s*\/check\b/.test(text);
  const { prose, code } = block
    ? { prose: block.question, code: block.code }
    : splitProse(fix ? text.replace(/^\s*\/check\s*/, '') : text);
  if (fix && !code) return <p className="ask-question">{text}</p>;
  if (!fix && !looksLikeCode(text))
    return <p className="ask-question">{text}</p>;
  return (
    <div className="ask-question ask-question-code">
      {prose && <p className="ask-question-prose">{prose}</p>}
      {fix && (
        <code className="ask-question-command">
          {block ? '/code' : '/check'}
        </code>
      )}
      <div className="ask-question-lines">
        <LineNumbers code={code} />
        <pre>
          <code>
            <Tokens
              code={code}
              language={detectLanguage(code, state.language)}
            />
          </code>
        </pre>
      </div>
    </div>
  );
}

function AskMessage({ message, progress }) {
  return (
    <li className="ask-message">
      <Question text={message.question} />
      {message.status === 'busy' && (
        <p className="ask-status">
          {progress > 0 && progress < 100
            ? `Loading the search model (first time only)… ${progress}%`
            : 'Searching…'}
        </p>
      )}
      {message.failure && (
        <p className="ask-status ask-rejected">{message.failure}</p>
      )}
      {message.result && <AskResult message={message} />}
    </li>
  );
}

const AskContext = createContext(null);

function withValue(result, owner, name, value) {
  if (result.values?.function === owner)
    return { ...result, values: { ...result.values, [name]: value } };
  return {
    ...result,
    templateValues: {
      ...result.templateValues,
      [owner]: { ...(result.templateValues?.[owner] ?? {}), [name]: value },
    },
  };
}

export function AskProvider({ children }) {
  const { state } = useDocumentation();
  const [isOpen, setIsOpen] = useState(false);
  const [messages, setMessages] = useState([]);
  const [progress, setProgress] = useState(0);
  const [modelFailed, setModelFailed] = useState(false);
  const askerRef = useRef({ key: null, asker: null });
  const apiFile = findApiFile(state.versions, state.version);
  const isBusy = messages.some((message) => message.status === 'busy');

  function updateMessage(id, change) {
    setMessages((current) =>
      current.map((message) =>
        message.id === id ? { ...message, ...change } : message,
      ),
    );
  }

  async function getAsker() {
    let encode = null;
    let classify = null;
    let fixNames = null;
    let scoreCode = null;
    try {
      [encode, classify] = await Promise.all([
        loadEncoder(setProgress),
        loadClassifier(),
      ]);
      setModelFailed(false);
    } catch (error) {
      console.error('Ask helper could not load its model', error);
      setModelFailed(true);
    }
    if (encode) {
      try {
        const layer = await loadLayer(models.nameFixer);
        fixNames = createNameFixer(
          createClassifier(layer.meta, layer.buffer),
          encode,
          state.document,
        );
      } catch (error) {
        console.error('Ask helper could not load the name-fixer', error);
      }
      try {
        const layer = await loadLayer(models.codeFaq);
        scoreCode = createCodeScorer(layer.meta, layer.buffer);
      } catch (error) {
        console.error('Ask helper could not load the code layer', error);
      }
    }
    const key = `${apiFile}:${encode ? 'model' : 'none'}`;
    const current = askerRef.current;
    if (current.key !== key || current.document !== state.document)
      askerRef.current = {
        key,
        document: state.document,
        asker: createAsker({
          encode,
          classify,
          fixNames,
          scoreCode,
          document: state.document,
        }),
      };
    return askerRef.current.asker;
  }

  async function askQuestion(asked, options = {}) {
    if (!asked || !state.document || isBusy) return;
    const id = Date.now();
    setIsOpen(true);
    setMessages([{ id, question: asked, status: 'busy' }]);
    try {
      const asker = await getAsker();
      updateMessage(id, {
        status: 'done',
        result: await asker(asked, state.language, options),
      });
    } catch (error) {
      updateMessage(id, { status: 'done', failure: error.message });
    }
  }

  async function chooseFunction(id, entryName) {
    const message = messages.find((item) => item.id === id);
    if (!message?.result || !askerRef.current.asker?.reread) return;
    const result = await askerRef.current.asker.reread(
      message.question,
      message.result,
      entryName,
      state.language,
    );
    updateMessage(id, { result });
  }

  function changeValue(id, owner, name, value) {
    setMessages((current) =>
      current.map((message) =>
        message.id === id
          ? {
              ...message,
              result: withValue(message.result, owner, name, value),
            }
          : message,
      ),
    );
  }

  function restart() {
    setMessages([]);
  }

  const value = {
    isOpen,
    setIsOpen,
    messages,
    progress,
    isBusy,
    modelFailed,
    askQuestion,
    chooseFunction,
    changeValue,
    restart,
    lastQuestion: messages[messages.length - 1]?.question ?? '',
  };
  return <AskContext.Provider value={value}>{children}</AskContext.Provider>;
}

function CommandMenu({ text, onPick, highlighted }) {
  const typed = text.slice(1).split(/\s/)[0].toLowerCase();
  const items = commandList.filter((item) => item.name.startsWith(typed));
  if (items.length === 0 || /\s/.test(text.trim())) return null;
  return (
    <ul className="ask-menu" role="listbox">
      {items.map((item, position) => (
        <li key={item.name}>
          <button
            type="button"
            role="option"
            aria-selected={position === highlighted}
            className={
              position === highlighted
                ? 'ask-menu-item ask-menu-active'
                : 'ask-menu-item'
            }
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => onPick(item)}
          >
            <code>{item.usage}</code>
            <span>{item.meaning}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}

function findCodePart(text) {
  const open = /[\/\\](code|check)\s*\{/.exec(text);
  if (!open) {
    const command = /^\s*\/check\b[^\n]*\n/.exec(text);
    return command
      ? { before: command[0], code: text.slice(command[0].length), after: '' }
      : null;
  }
  const start = open.index + open[0].length;
  let depth = 1;
  let end = start;
  for (; end < text.length && depth > 0; end += 1) {
    if (text[end] === '{') depth += 1;
    if (text[end] === '}') depth -= 1;
  }
  const close = depth === 0 ? end - 1 : text.length;
  return {
    before: text.slice(0, start),
    code: text.slice(start, close),
    after: text.slice(close),
  };
}

function ShownInput({ text, language, innerRef }) {
  const part = findCodePart(text);
  if (!part) return null;
  return (
    <pre
      ref={innerRef}
      className="ask-input ask-input-shown"
      aria-hidden="true"
    >
      {part.before}
      <Tokens code={part.code} language={detectLanguage(part.code, language)} />
      {part.after}
      {'\n'}
    </pre>
  );
}

function AskInput({ className, autoFocus }) {
  const { state } = useDocumentation();
  const { isBusy, askQuestion, lastQuestion } = useContext(AskContext);
  const [question, setQuestion] = useState('');
  const [highlighted, setHighlighted] = useState(0);
  const showsMenu = question.startsWith('/') && !/\s/.test(question.trim());
  const menuItems = showsMenu
    ? commandList.filter((item) =>
        item.name.startsWith(question.slice(1).toLowerCase()),
      )
    : [];
  const isCode =
    /^\/check\b|[\/\\](code|check)\s*\{/.test(question) ||
    looksLikeCode(question);
  const maxLength = isCode ? limits.code : limits.question;
  const isTooLong = question.length > maxLength;
  const inputRef = useRef(null);
  const shownRef = useRef(null);
  const isColored = Boolean(findCodePart(question));

  useEffect(() => {
    const input = inputRef.current;
    if (!input) return;
    input.style.height = 'auto';
    input.style.height = `${input.scrollHeight}px`;
  }, [question]);

  function submit(event) {
    event.preventDefault();
    const asked = question.trim();
    if (!asked || isBusy || isTooLong) return;
    setQuestion('');
    askQuestion(asked);
  }

  function pick(item) {
    setQuestion(
      ['convert', 'check'].includes(item.name)
        ? `/${item.name}`
        : `/${item.name} `,
    );
    setHighlighted(0);
  }

  function onKeyDown(event) {
    const picksCommand =
      showsMenu &&
      menuItems.length > 0 &&
      question !== `/${menuItems[highlighted]?.name}`;
    if (
      event.key === 'Enter' &&
      !event.shiftKey &&
      !event.nativeEvent.isComposing &&
      !picksCommand
    ) {
      event.preventDefault();
      submit(event);
      return;
    }
    if (!showsMenu || menuItems.length === 0) return;
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setHighlighted((current) => (current + 1) % menuItems.length);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setHighlighted(
        (current) => (current - 1 + menuItems.length) % menuItems.length,
      );
    } else if (
      event.key === 'Tab' ||
      (event.key === 'Enter' && question !== `/${menuItems[highlighted].name}`)
    ) {
      event.preventDefault();
      pick(menuItems[highlighted]);
    }
  }

  return (
    <div className="ask-input-holder">
      {showsMenu && (
        <CommandMenu
          text={question}
          onPick={pick}
          highlighted={Math.min(highlighted, Math.max(menuItems.length - 1, 0))}
        />
      )}
      <form
        className={`ask-field ${className}${question.includes('\n') ? ' ask-field-multi' : ''}`}
        onSubmit={submit}
      >
        <div className="ask-input-holder-code">
          <ShownInput
            text={question}
            language={state.language}
            innerRef={shownRef}
          />
          <textarea
            ref={inputRef}
            rows={1}
            className={`ask-input${isCode ? ' ask-input-code' : ''}${isColored ? ' ask-input-colored' : ''}`}
            spellCheck={!isColored}
            onScroll={(event) => {
              if (shownRef.current)
                shownRef.current.scrollTop = event.target.scrollTop;
            }}
            placeholder="Ask, paste a snippet, or type / for commands…"
            aria-label="Ask a usage question"
            maxLength={limits.code}
            value={question}
            autoFocus={autoFocus}
            onChange={(event) => {
              setQuestion(event.target.value);
              setHighlighted(0);
            }}
            onKeyDown={onKeyDown}
          />
        </div>
        <span
          className={isTooLong ? 'ask-count ask-rejected' : 'ask-count'}
          aria-live="polite"
        >
          {question.length}/{maxLength}
        </span>
        {!question && lastQuestion && !isBusy ? (
          <button
            key="edit"
            type="button"
            className="ask-button"
            title="Change your question and ask again"
            onClick={(event) => {
              event.preventDefault();
              setQuestion(lastQuestion);
              inputRef.current?.focus();
            }}
          >
            Edit
          </button>
        ) : (
          <button
            key="ask"
            type="submit"
            className="ask-button"
            disabled={isBusy || !question.trim() || isTooLong}
          >
            {isBusy ? '…' : 'Ask'}
          </button>
        )}
      </form>
    </div>
  );
}

function AgentIcon() {
  return (
    <svg
      width="26"
      height="26"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M12 3v3" />
      <circle cx="12" cy="2.5" r="0.8" fill="currentColor" />
      <rect x="4" y="6" width="16" height="12" rx="4" />
      <circle cx="9" cy="12" r="1.3" fill="currentColor" stroke="none" />
      <circle cx="15" cy="12" r="1.3" fill="currentColor" stroke="none" />
      <path d="M9.5 15.5h5" />
      <path d="M2 11v3M22 11v3" />
    </svg>
  );
}

const sizeKey = 'ask-panel-size';
const smallest = { width: 360, height: 360 };
const resizeStep = 24;

function readSize() {
  try {
    const size = JSON.parse(localStorage.getItem(sizeKey));
    return size && Number.isFinite(size.width) && Number.isFinite(size.height)
      ? size
      : null;
  } catch {
    return null;
  }
}

function saveSize(size) {
  try {
    if (size) localStorage.setItem(sizeKey, JSON.stringify(size));
    else localStorage.removeItem(sizeKey);
  } catch {}
}

function fitSize(size) {
  return {
    width: Math.round(
      Math.min(Math.max(size.width, smallest.width), window.innerWidth - 32),
    ),
    height: Math.round(
      Math.min(Math.max(size.height, smallest.height), window.innerHeight - 48),
    ),
  };
}

function usePanelSize(panel) {
  const [size, setSize] = useState(readSize);

  function change(next) {
    const fitted = next ? fitSize(next) : null;
    setSize(fitted);
    saveSize(fitted);
  }

  function startDrag(event) {
    if (!panel.current) return;
    event.preventDefault();
    const box = panel.current.getBoundingClientRect();
    const start = {
      x: event.clientX,
      y: event.clientY,
      width: box.width,
      height: box.height,
    };
    const handle = event.currentTarget;
    handle.setPointerCapture(event.pointerId);
    function move(moveEvent) {
      change({
        width: start.width + start.x - moveEvent.clientX,
        height: start.height + start.y - moveEvent.clientY,
      });
    }
    function stop() {
      handle.removeEventListener('pointermove', move);
      handle.removeEventListener('pointerup', stop);
      handle.removeEventListener('pointercancel', stop);
    }
    handle.addEventListener('pointermove', move);
    handle.addEventListener('pointerup', stop);
    handle.addEventListener('pointercancel', stop);
  }

  function onKeyDown(event) {
    if (!panel.current) return;
    const box = panel.current.getBoundingClientRect();
    const steps = {
      ArrowLeft: [resizeStep, 0],
      ArrowRight: [-resizeStep, 0],
      ArrowUp: [0, resizeStep],
      ArrowDown: [0, -resizeStep],
    };
    const step = steps[event.key];
    if (!step) return;
    event.preventDefault();
    change({ width: box.width + step[0], height: box.height + step[1] });
  }

  return { size, startDrag, onKeyDown, reset: () => change(null) };
}

export function AskChat() {
  const {
    isOpen,
    setIsOpen,
    messages,
    progress,
    modelFailed,
    restart,
    askQuestion,
  } = useContext(AskContext);
  const listEnd = useRef(null);
  const panel = useRef(null);
  const resize = usePanelSize(panel);

  useEffect(() => {
    if (!isOpen) return undefined;
    function closeOutside(event) {
      if (!panel.current || panel.current.contains(event.target)) return;
      if (event.target.closest?.('[role="tablist"][aria-label="Language"]'))
        return;
      setIsOpen(false);
    }
    document.addEventListener('pointerdown', closeOutside);
    return () => document.removeEventListener('pointerdown', closeOutside);
  }, [isOpen, setIsOpen]);

  useEffect(() => {
    listEnd.current?.scrollIntoView({ block: 'end' });
  }, [messages, isOpen]);

  if (!isOpen) {
    return (
      <button
        type="button"
        className="ask-open"
        aria-label="Open the Quant·ion assistant"
        title="Ask about Quant·ion"
        onClick={() => setIsOpen(true)}
      >
        <AgentIcon />
      </button>
    );
  }

  return (
    <section
      ref={panel}
      className="ask-panel"
      aria-label="Ask about Quant·ion"
      style={
        resize.size
          ? {
              width: resize.size.width,
              height: resize.size.height,
              maxHeight: 'none',
            }
          : undefined
      }
    >
      <button
        type="button"
        className="ask-resize"
        aria-label="Resize the chat: drag, or use the arrow keys. Double-click to reset."
        title="Drag to resize · double-click to reset"
        onPointerDown={resize.startDrag}
        onKeyDown={resize.onKeyDown}
        onDoubleClick={resize.reset}
      />
      <header className="ask-header">
        <span className="ask-title">
          <AgentIcon />
          Ask about Quant·ion
        </span>
        <button
          type="button"
          className="ask-restart"
          onClick={restart}
          disabled={messages.length === 0}
        >
          Try another question
        </button>
        <button
          type="button"
          className="ask-close"
          aria-label="Close"
          onClick={() => setIsOpen(false)}
        >
          ×
        </button>
      </header>
      <ul className="ask-messages">
        {messages.length === 0 && (
          <li className="ask-hint">
            <HelperNotice />
            <p>Ask, or pick one:</p>
            <QuestionChips items={getStarters()} />
            <p>Or type a command:</p>
            <ul className="ask-meanings ask-commands">
              {commandList.map((item) => (
                <li key={item.name}>
                  <button
                    type="button"
                    className="ask-command-link"
                    onClick={() => askQuestion(`/${item.name}`)}
                  >
                    <code>{item.usage}</code>
                  </button>{' '}
                  {item.meaning}
                </li>
              ))}
            </ul>
            <p>
              Or ask about your code: write the question, then the code inside{' '}
              <code>/code{'{ … }'}</code>.
            </p>
            <CodeExample />
            <p className="ask-tip">
              Questions with enough context get better answers.
            </p>
          </li>
        )}
        {modelFailed && <li className="ask-hint">{modelFailure}</li>}
        {messages.map((message) => (
          <AskMessage key={message.id} message={message} progress={progress} />
        ))}
        <li ref={listEnd} aria-hidden="true" />
      </ul>
      <AskInput className="ask-chat-input" autoFocus />
      <p className="ask-warning">
        Quant·ion usage only. Not perfect: provide enough context and
        double-check the answers.
      </p>
    </section>
  );
}
