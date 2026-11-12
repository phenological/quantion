export const models = {
  encoder: { name: 'helper-answer-21', dtype: 'q8', pooling: 'mean' },
  answers: { labels: 'answers.json', weights: 'answers.bin' },
  nameFixer: { labels: 'name-fixer.json', weights: 'name-fixer.bin' },
  codeFaq: { labels: 'code-faq.json', weights: 'code-faq.bin' },
  folder: 'models/',
};

export const limits = {
  question: 250,
  code: 1000,
  answer: 1000,
  entries: 3,
  mainKeyBonus: 0.1,
  highProbability: 0.5,
  lowProbability: 0.2,
  builtinGuess: 0.7,
  keepOverCode: 0.8,
  faqSureWithCode: 0.2,
  commonWordShare: 0.05,
};

export const refusal =
  "I only have knowledge about Quant·ion. I'm not sure what you're asking for, sorry!";

export const issuesLink = 'https://github.com/phenological/quantion/issues';
