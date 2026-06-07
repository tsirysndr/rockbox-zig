import { atom } from "jotai";
import { SearchResults } from "../../Hooks/GraphQL";

export const filterState = atom<{
  term: string;
  results?: SearchResults;
}>({
  term: "",
  results: undefined,
});
