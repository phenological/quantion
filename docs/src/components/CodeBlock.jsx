import { useState } from "react";

import { splitIntoTokens } from "../highlight";

function getCopyLabel(copied) {
  if (copied) return "Copied";
  return "Copy";
}

export function CodeBlock({ title, code, language }) {
  const [copied, setCopied] = useState(false);

  function copyCode() {
    navigator.clipboard.writeText(code).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    });
  }

  return (
    <section className="block">
      <h3>
        {title}
        <button type="button" className="copy" onClick={copyCode}>
          {getCopyLabel(copied)}
        </button>
      </h3>
      <pre>
        {splitIntoTokens(code, language).map((token, index) =>
          token.kind ? (
            <span key={index} className={`token-${token.kind}`}>
              {token.text}
            </span>
          ) : (
            token.text
          ),
        )}
      </pre>
    </section>
  );
}
