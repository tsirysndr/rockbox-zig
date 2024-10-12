import { FC, useEffect, useState } from "react";
import AlbumDetails from "./AlbumDetails";
import { useNavigate, useParams } from "react-router-dom";
import { useGetAlbumQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { Track } from "../../Types/track";

const AlbumDetailsWithData: FC = () => {
  const [volumes, setVolumes] = useState<Track[][]>([]);
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
        artistId: x.artistId!,
        time: formatTime(x.length),
        discnum: x.discnum,
      })) || []
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loading]);

  useEffect(() => {
    refetch();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  useEffect(() => {
    if (loading || !tracks.length) {
      return;
    }
    getVolumes();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tracks, loading]);

  function getVolumes() {
    if (!tracks.some((track) => track.discnum === 2)) {
      return;
    }
    let volume = 1;
    while (tracks.some((track) => track.discnum === volume)) {
      volumes.push(tracks.filter((track) => track.discnum === volume));
      setVolumes(volumes);
      volume++;
    }
    if (volumes.length > 1) {
      setTracks([]);
    }
  }

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
      volumes={volumes}
    />
  );
};

export default AlbumDetailsWithData;
