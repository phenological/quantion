import { reviewCode } from './review.js';

export const commandList = [
  {
    name: 'check',
    usage: '/check',
    meaning: 'Finds syntax mistakes in your code.',
  },
  {
    name: 'convert',
    usage: '/convert',
    meaning: 'Converts a file here: .ion to mzML, or mzML to .ion.',
  },
];

const exampleQuestions = [
  'How do I find peaks above intensity 1000?',
  'What is a target?',
  'I have mz and rt, what do I use?',
];

export function isCommand(text) {
  return /^\s*\//.test(text);
}

export function runCommand(text, document, language) {
  const match = /^\s*\/(\w*)\s*([\s\S]*)$/.exec(text);
  if (!match) return null;
  const [, name, rest] = match;
  const argument = rest.trim();
  if (name === '')
    return {
      command: 'help',
      commands: commandList,
      examples: exampleQuestions,
    };
  if (name === 'convert') return { command: 'convert' };
  if (name === 'check') {
    if (!argument) return { command: 'check', open: true };
    const review = reviewCode(argument, document, language);
    if (review.entries.length === 0 && review.notes.length === 0)
      return {
        command: 'check',
        failure: 'I found no Quant·ion function in that code.',
      };
    return { command: 'check', review };
  }
  return {
    command: 'unknown',
    name,
    commands: commandList,
    failure: `There is no command \`/${name}\`.`,
  };
}
