import { FC, useEffect, useState } from "react";
import Albums from "./Albums";
import { useGetAlbumsQuery } from "../../Hooks/GraphQL";

const AlbumsWithData: FC = () => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const [albums, setAlbums] = useState<any[]>([]);
  const { data } = useGetAlbumsQuery();

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
    />
  );
};

export default AlbumsWithData;
