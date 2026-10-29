import { actions } from "../state/documentationState";
import { useDocumentation } from "../state/DocumentationContext";

function groupEntries(entries) {
  const groups = new Map();
  for (const entry of entries) {
    if (!groups.has(entry.group)) groups.set(entry.group, []);
    groups.get(entry.group).push(entry.name);
  }
  return [...groups];
}

function SidebarButton({ name, label }) {
  const { state, dispatch } = useDocumentation();
  return (
    <button
      type="button"
      aria-current={state.entryName === name}
      onClick={() => dispatch({ type: actions.entryChosen, entryName: name })}
    >
      {label ?? name}
    </button>
  );
}

export function Sidebar() {
  const { state } = useDocumentation();
  return (
    <nav className="sidebar" aria-label="Functions and types">
      <div className="group">
        <SidebarButton name="overview" label="Overview" />
      </div>
      {groupEntries(state.document.entries).map(([group, names]) => (
        <div className="group" key={group}>
          <h2>{group}</h2>
          {names.map((name) => (
            <SidebarButton key={name} name={name} />
          ))}
        </div>
      ))}
    </nav>
  );
}
