import { actions } from "../state/documentationState";
import { useDocumentation } from "../state/DocumentationContext";

export function TypeText({ text }) {
  const { state, dispatch } = useDocumentation();
  const typeNames = new Set(state.document.entries.filter((entry) => entry.kind === "type").map((entry) => entry.name));
  const parts = text.split(/([A-Z][A-Za-z]+)/);
  return parts.map((part, index) =>
    typeNames.has(part) ? (
      <button key={index} type="button" className="type-link" onClick={() => dispatch({ type: actions.entryChosen, entryName: part })}>
        {part}
      </button>
    ) : (
      part
    ),
  );
}
