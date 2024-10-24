import { FC, useMemo } from "react";
import Artists from "./Artists";
import { useGetArtistsQuery } from "../../Hooks/GraphQL";
import { useRecoilValue } from "recoil";
import { filterState } from "../Filter/FilterState";

const ArtistsWithData: FC = () => {
  const filter = useRecoilValue(filterState);
  const { data, loading } = useGetArtistsQuery();
  const artists = useMemo(() => {
    if (filter.term.length > 0 && filter.results) {
      return (filter.results?.artists || []).map((x) => ({
        id: x.id,
        name: x.name,
      }));
    }
    return (data?.artists || []).map((x) => ({
      id: x.id,
      name: x.name,
    }));
  }, [data, filter]);

  return (
    <Artists
      onFilter={() => {}}
      onClickArtist={() => {}}
      artists={artists}
      keyword={filter.term}
      loading={loading}
    />
  );
};

export default ArtistsWithData;
