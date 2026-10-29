import { produce } from "immer";

export const initialState = {
  versions: [],
  version: null,
  document: null,
  language: "python",
  entryName: "overview",
  status: "loading",
};

export const actions = {
  versionsLoaded: "versionsLoaded",
  versionChosen: "versionChosen",
  documentLoaded: "documentLoaded",
  documentFailed: "documentFailed",
  languageChosen: "languageChosen",
  entryChosen: "entryChosen",
};

export const documentationReducer = produce((state, action) => {
  switch (action.type) {
    case actions.versionsLoaded:
      state.versions = action.versions;
      state.version = action.latest;
      break;
    case actions.versionChosen:
      state.version = action.version;
      state.status = "loading";
      break;
    case actions.documentLoaded:
      state.document = action.document;
      state.status = "ready";
      if (!findEntry(action.document, state.entryName)) state.entryName = "overview";
      break;
    case actions.documentFailed:
      state.status = "failed";
      break;
    case actions.languageChosen:
      state.language = action.language;
      break;
    case actions.entryChosen:
      state.entryName = action.entryName;
      break;
  }
});

export function findApiFile(versions, version) {
  return versions.find((item) => item.version === version)?.api ?? null;
}

export function findEntry(document, entryName) {
  return document?.entries.find((entry) => entry.name === entryName) ?? null;
}
