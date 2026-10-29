import { TypeText } from "./TypeText";

export function ReturnsLine({ returns, language }) {
  const type = returns?.languages[language];
  if (!type) return null;
  return (
    <section className="block">
      <h3>Returns</h3>
      <p className="returns">
        <span className="type">
          <TypeText text={type} />
        </span>
        <span>{returns.meaning}</span>
      </p>
    </section>
  );
}
