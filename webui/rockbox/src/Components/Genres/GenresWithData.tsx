import { FC } from "react";
import { useGetGenresQuery } from "../../Hooks/useGenres";
import Genres from "./Genres";

const GenresWithData: FC = () => {
  const { data, isLoading } = useGetGenresQuery();
  return <Genres genres={data?.genres ?? []} loading={isLoading} />;
};

export default GenresWithData;
