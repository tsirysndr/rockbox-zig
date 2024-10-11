import { FC, useEffect, useState } from "react";
import AlbumDetails from "./AlbumDetails";
import { useNavigate, useParams } from "react-router-dom";
import { useGetAlbumQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { Track } from "../../Types/track";

const AlbumDetailsWithData: FC = () => {
  const [tracks, setTracks] = useState<Track[]>([]);
  const { formatTime } = useTimeFormat();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const { data, loading, refetch } = useGetAlbumQuery({
    variables: {
      id: id!,
    },
  });
  const album = data
    ? {
        ...data.album!,
        albumArt: data.album?.albumArt
          ? `http://localhost:6062/covers/${data.album?.albumArt}`
          : "",
      }
    : null;
  useEffect(() => {
    if (loading || !album) {
      return;
    }
    setTracks(
      album.tracks.map((x) => ({
        id: x.id!,
        trackNumber: x.tracknum,
        title: x.title,
        artist: x.artist,
        time: formatTime(x.length),
      })) || []
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loading]);

  useEffect(() => {
    refetch();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  return (
    <AlbumDetails
      onGoBack={() => navigate(-1)}
      onLike={() => {}}
      onPlayAll={() => {}}
      onShuffleAll={() => {}}
      onUnlike={() => {}}
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      tracks={tracks as any[]}
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      album={album as any}
    />
  );
};

export default AlbumDetailsWithData;
