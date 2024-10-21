import { FC, useEffect, useState } from "react";
import Tracks from "./Tracks";
import { usePlayAllTracksMutation, useTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { Track } from "../../Types/track";

const TracksWithData: FC = () => {
  const { data, loading } = useTracksQuery();
  const [tracks, setTracks] = useState<Track[]>([]);
  const { formatTime } = useTimeFormat();
  const [playAllTracks] = usePlayAllTracksMutation();

  useEffect(() => {
    if (!data || loading) {
      return;
    }

    setTracks(
      data.tracks.map((x, i) => ({
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
        path: x.path,
      }))
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, loading]);

  const onPlayTrack = (position: number) => {
    playAllTracks({
      variables: {
        position,
      },
    });
  };

  return <Tracks tracks={tracks} onPlayTrack={onPlayTrack} />;
};

export default TracksWithData;
