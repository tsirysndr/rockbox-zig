import { FC, useEffect, useState } from "react";
import Albums from "./Albums";
import { useGetAlbumsQuery } from "../../Hooks/GraphQL";
import { useRecoilValue } from "recoil";
import { filterState } from "../Filter/FilterState";

const AlbumsWithData: FC = () => {
  const filter = useRecoilValue(filterState);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [albums, setAlbums] = useState<any[]>([]);
  const { data, loading } = useGetAlbumsQuery();

  useEffect(() => {
    if (filter.term.length > 0 && filter.results) {
      setAlbums(
        filter.results.albums.map((x) => ({
          id: x.id,
          title: x.title,
          artist: x.artist,
          cover: x.albumArt ? `http://localhost:6062/covers/${x.albumArt}` : "",
          year: x.year,
          artistId: x.artistId,
        }))
      );
      return;
    }
    if (data) {
      setAlbums(
        data.albums.map((x) => ({
          id: x.id,
          title: x.title,
          artist: x.artist,
          cover: x.albumArt ? `http://localhost:6062/covers/${x.albumArt}` : "",
          year: x.year,
          artistId: x.artistId,
        }))
      );
    }
  }, [filter, data]);

  useEffect(() => {
    if (data) {
      setAlbums(
        data.albums.map((x) => ({
          id: x.id,
          title: x.title,
          artist: x.artist,
          cover: x.albumArt ? `http://localhost:6062/covers/${x.albumArt}` : "",
          year: x.year,
          artistId: x.artistId,
        }))
      );
    }
  }, [data]);

  return (
    <Albums
      onFilter={() => {}}
      albums={albums}
      onLike={() => {}}
      onUnLike={() => {}}
      keyword={filter.term}
      loading={loading}
    />
  );
};

export default AlbumsWithData;
