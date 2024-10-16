import { FC, useMemo } from "react";
import Artists from "./Artists";
import { useGetArtistsQuery } from "../../Hooks/GraphQL";

const ArtistsWithData: FC = () => {
  const { data } = useGetArtistsQuery();
  const artists = useMemo(() => {
    return (data?.artists || []).map((x) => ({
      id: x.id,
      name: x.name,
    }));
  }, [data]);

  return (
    <Artists onFilter={() => {}} onClickArtist={() => {}} artists={artists} />
  );
};

export default ArtistsWithData;
