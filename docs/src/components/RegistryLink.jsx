const registries = {
  rust: { name: "crates.io", address: "https://crates.io/crates/quantion" },
  python: { name: "PyPI", address: "https://pypi.org/project/quantion/" },
};

export function RegistryLink({ language }) {
  const registry = registries[language];
  if (!registry) return null;
  return (
    <a className="registry-link" href={registry.address} target="_blank" rel="noreferrer">
      {registry.name}
    </a>
  );
}
