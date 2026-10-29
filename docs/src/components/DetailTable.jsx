import { TypeText } from "./TypeText";

export function DetailTable({ title, rows, language }) {
  const visibleRows = rows.filter((row) => row.languages[language]);
  if (!visibleRows.length) return null;
  const hasDefault = visibleRows.some((row) => row.languages[language].default !== undefined);
  return (
    <section className="block">
      <h3>{title}</h3>
      <div className="table-wrap">
        <table className="details">
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              {hasDefault && <th>Default</th>}
              <th>Meaning</th>
            </tr>
          </thead>
          <tbody>
            {visibleRows.map((row) => {
              const cell = row.languages[language];
              return (
                <tr key={cell.name}>
                  <td className="name">{cell.name}</td>
                  <td className="type">
                    <TypeText text={cell.type} />
                  </td>
                  {hasDefault && <td className="default">{cell.default ?? "—"}</td>}
                  <td>{row.meaning}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}
