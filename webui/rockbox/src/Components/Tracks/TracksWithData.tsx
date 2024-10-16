import { FC, useMemo } from "react";
import Tracks from "./Tracks";
import { useTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";

const TracksWithData: FC = () => {
  const { data } = useTracksQuery();
  const { formatTime } = useTimeFormat();
  const tracks = useMemo(() => {
    if (!data) {
      return [];
    }

    return data.tracks.map((x, i) => ({
      id: x.id!,
      trackNumber: i + 1,
      title: x.title,
      artist: x.artist,
      album: x.album,
      time: formatTime(x.length),
      albumArt: x.albumArt
        ? `http://localhost:6062/covers/${x.albumArt}`
        : undefined,
      albumId: x.albumId,
      artistId: x.artistId,
    }));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data]);

  const onPlayTrack = (trackId: string) => {
    console.log(">>", trackId);
  };

  return <Tracks tracks={tracks} onPlayTrack={onPlayTrack} />;
};

export default TracksWithData;
