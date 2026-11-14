import { useState } from 'react';
import { docsAddress } from '../site';
import { useDocumentation } from '../state/DocumentationContext';
import { CopyButton } from './CopyButton';

function SparkIcon() {
  return (
    <svg
      viewBox="0 0 16 16"
      width="14"
      height="14"
      aria-hidden="true"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinejoin="round"
    >
      <path d="M8 1.5l1.6 4.9 4.9 1.6-4.9 1.6L8 14.5l-1.6-4.9L1.5 8l4.9-1.6z" />
    </svg>
  );
}

function LlmAddress() {
  const { state } = useDocumentation();
  if (!state.version) return null;
  const address = docsAddress(state.version);
  return (
    <div className="llm-link-address">
      <a href={address} target="_blank" rel="noreferrer">
        {address}
      </a>
      <CopyButton text={address} label="Copy the link" />
    </div>
  );
}

const RELEASES =
  'https://github.com/phenological/quantion/releases/latest/download';
const MCP_GUIDE = 'https://github.com/phenological/quantion/tree/main/mcp';

const SYSTEMS = {
  unix: {
    label: 'macOS / Linux',
    install: `curl -fsSL ${RELEASES}/install.sh | sh`,
    command: '~/.local/bin/quantion-mcp --allow /path/to/data',
  },
  windows: {
    label: 'Windows',
    install: `irm ${RELEASES}/install.ps1 | iex`,
    command:
      '"$env:LOCALAPPDATA\\Programs\\quantion-mcp\\quantion-mcp.exe" --allow C:\\path\\to\\data',
  },
};

function CommandBox({ command }) {
  return (
    <div className="llm-link-address">
      <code>{command}</code>
      <CopyButton text={command} label="Copy the command" />
    </div>
  );
}

function McpSetup() {
  const [system, setSystem] = useState('unix');
  const commands = SYSTEMS[system];
  return (
    <>
      <p>
        Apps that support MCP can also run Quant·ion on your computer with
        quantion-mcp. It reads only the folders you allow.
      </p>
      <div className="ask-own-ai-systems" role="group" aria-label="System">
        {Object.entries(SYSTEMS).map(([key, item]) => (
          <button
            key={key}
            type="button"
            aria-pressed={system === key}
            onClick={() => setSystem(key)}
          >
            {item.label}
          </button>
        ))}
      </div>
      <p className="ask-own-ai-step">Install it:</p>
      <CommandBox command={commands.install} />
      <p className="ask-own-ai-step">
        Then add it to your app as an MCP server with this command:
      </p>
      <CommandBox command={commands.command} />
      <p className="ask-own-ai-step">
        More in the{' '}
        <a href={MCP_GUIDE} target="_blank" rel="noreferrer">
          MCP guide
        </a>
        .
      </p>
    </>
  );
}

export function HelperNotice() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="ask-notice" role="note">
      <p>
        This helper only answers about how to use the Quant·ion toolkit, one
        question at a time.
      </p>
      <button
        type="button"
        className="ask-own-ai-toggle"
        aria-expanded={isOpen}
        onClick={() => setIsOpen(!isOpen)}
      >
        <SparkIcon />
        Need a longer conversation? Use your AI assistant
        <span className="ask-own-ai-arrow" aria-hidden="true">
          {isOpen ? '▴' : '▾'}
        </span>
      </button>
      {isOpen && (
        <div className="ask-own-ai">
          <p>
            This helper does not remember what you asked before. For longer
            work, give your AI assistant the documentation as one text file:
          </p>
          <LlmAddress />
          <McpSetup />
        </div>
      )}
    </div>
  );
}
