import { splitIntoTokens } from '../highlight';
import { CopyButton } from './CopyButton';

export function CodeBlock({ title, code, language, numbered = false }) {
  return (
    <section className="block">
      <h3>
        {title}
        <CopyButton text={code} label={`Copy ${title}`} />
      </h3>
      <div className={numbered ? 'ask-question-lines' : undefined}>
        {numbered && (
          <pre className="ask-line-numbers" aria-hidden="true">
            {code
              .split('\n')
              .map((_, index) => index + 1)
              .join('\n')}
          </pre>
        )}
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
      </div>
    </section>
  );
}
