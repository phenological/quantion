import { findEntry } from "../state/documentationState";
import { useDocumentation } from "../state/DocumentationContext";
import { CodeBlock } from "./CodeBlock";
import { DetailTable } from "./DetailTable";
import { GitHubIcon } from "./GitHubLinks";
import { ReturnsLine } from "./ReturnsLine";

function getDisplayName(entry, language) {
  if (entry.kind !== "function" || language !== "js") return entry.name;
  return entry.name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

function EntryHeading({ title, group, summary }) {
  return (
    <>
      <div className="entry-heading">
        <h2>{title}</h2>
        {group && <span className="badge">{group}</span>}
      </div>
      <p className="summary">{summary}</p>
    </>
  );
}

function Overview({ overview, language }) {
  const languageLabel = useDocumentation().state.document.languages.find((item) => item.key === language).label;
  return (
    <>
      <EntryHeading title={overview.title} summary={overview.summary} />
      <CodeBlock title={`Install for ${languageLabel}`} code={overview.install[language]} language={language === "r" ? "r" : "shell"} />
      <section className="block">
        <h3>Projects</h3>
        <div className="projects">
          {overview.projects.map((project) => (
            <a key={project.name} className="project" href={project.address} target="_blank" rel="noreferrer">
              <span className="project-name">
                <GitHubIcon />
                {project.name}
              </span>
              <span className="project-description">{project.description}</span>
            </a>
          ))}
        </div>
      </section>
    </>
  );
}

function Entry({ entry, language }) {
  const languageLabel = useDocumentation().state.document.languages.find((item) => item.key === language).label;
  const code = entry.languages[language];
  const isType = entry.kind === "type";
  return (
    <>
      <EntryHeading title={getDisplayName(entry, language)} group={entry.group} summary={entry.summary} />
      {entry.warning && (
        <p className="warning" role="note">
          <strong>Warning: </strong>
          {entry.warning}
        </p>
      )}
      {!code ? (
        <p className="missing">{`${entry.name} is not part of the ${languageLabel} API.`}</p>
      ) : (
        <>
          <CodeBlock title={`${languageLabel} ${isType ? "definition" : "signature"}`} code={code.signature} language={language} />
          {isType ? (
            <DetailTable title="Fields" rows={entry.fields} language={language} />
          ) : (
            <>
              <DetailTable title="Arguments" rows={entry.arguments} language={language} />
              <ReturnsLine returns={entry.returns} language={language} />
            </>
          )}
          {code.example && <CodeBlock title={`${languageLabel} example`} code={code.example} language={language} />}
        </>
      )}
    </>
  );
}

export function EntryView() {
  const { state } = useDocumentation();
  const entry = findEntry(state.document, state.entryName);
  return entry ? <Entry entry={entry} language={state.language} /> : <Overview overview={state.document.overview} language={state.language} />;
}
