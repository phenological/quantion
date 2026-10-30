const ionicAddress = "https://github.com/phenological/ionic";

export function IonicText({ text }) {
  const parts = text.split(/\b(Ionic)\b/);
  const output = [];
  for (let index = 0; index < parts.length; index++) {
    const part = parts[index];
    const isIonic = index % 2 === 1;
    if (isIonic) {
      output.push(
        <a key={index} className="prose-link" href={ionicAddress} target="_blank" rel="noreferrer">
          {part}
        </a>,
      );
    } else if (part) {
      output.push(part);
    }
  }
  return output;
}
