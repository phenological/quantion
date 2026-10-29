import { useDocumentation } from "../state/DocumentationContext";

export function VersionsView() {
  const { state } = useDocumentation();

  return (
    <>
      <div className="entry-heading">
        <h2>Versions</h2>
      </div>
      <p className="summary">Every published version, newest first, with its release date and commit message.</p>
      <ul className="version-history">
        {state.versions.map((item) => (
          <li key={item.version}>
            <span className="version-history-number">{item.version}</span>
            <span className="version-history-date">{item.date ?? "Not released yet"}</span>
            {item.message && <span className="version-history-message">{item.message}</span>}
          </li>
        ))}
      </ul>
    </>
  );
}
