import { atom } from "recoil";
import { SearchResults } from "../../Hooks/GraphQL";

export const filterState = atom<{
  term: string;
  results?: SearchResults;
}>({
  key: "filterState",
  default: {
    term: "",
    results: undefined,
  },
});
