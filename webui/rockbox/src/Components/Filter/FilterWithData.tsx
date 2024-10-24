import { FC } from "react";
import Filter from "./Filter";
import { useSearchLazyQuery } from "../../Hooks/GraphQL";
import { useRecoilState } from "recoil";
import { filterState } from "./FilterState";
import _ from "lodash";

const FilterWithData: FC<{ placeholder?: string }> = (props) => {
  const [, setFilterState] = useRecoilState(filterState);
  const [search] = useSearchLazyQuery();
  const onSearch = async (term: string) => {
    setFilterState((state) => ({ ...state, term }));
    if (term.length > 2) {
      _.debounce(async () => {
        const { data } = await search({ variables: { term } });
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        setFilterState((state) => ({ ...state, results: data?.search as any }));
      }, 500)();
    }
    if (term.length === 0) {
      setFilterState({ term: "", results: undefined });
    }
  };
  return <Filter onChange={onSearch} {...props} />;
};

export default FilterWithData;
