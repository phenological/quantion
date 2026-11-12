import { useState } from 'react';
import { llmsAddress } from '../site';
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
  return (
    <div className="llm-link-address">
      <a href={llmsAddress} target="_blank" rel="noreferrer">
        {llmsAddress}
      </a>
      <CopyButton text={llmsAddress} label="Copy the link" />
    </div>
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
        Need a longer conversation? Use your favorite AI assistant
        <span className="ask-own-ai-arrow" aria-hidden="true">
          {isOpen ? '▴' : '▾'}
        </span>
      </button>
      {isOpen && (
        <div className="ask-own-ai">
          <p>
            This helper does not remember what you asked before. For longer
            work, use your favorite AI assistant (Claude, ChatGPT, Cursor).
            Share this link with it and it can read the whole Quant·ion
            documentation.
          </p>
          <LlmAddress />
        </div>
      )}
    </div>
  );
}
