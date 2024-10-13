import { FC, useEffect, useState } from "react";
import ControlBar from "./ControlBar";
import {
  useGetCurrentTrackQuery,
  useGetPlaybackStatusQuery,
  useNextMutation,
  usePauseMutation,
  usePreviousMutation,
  useResumeMutation,
} from "../../Hooks/GraphQL";
import { CurrentTrack } from "../../Types/track";

const ControlBarWithData: FC = () => {
  const [nowPlaying, setNowPlaying] = useState<CurrentTrack | undefined>(
    undefined
  );
  const { data, loading } = useGetCurrentTrackQuery();
  const { data: playback } = useGetPlaybackStatusQuery();
  const [pause] = usePauseMutation();
  const [resume] = useResumeMutation();
  const [previous] = usePreviousMutation();
  const [next] = useNextMutation();

  useEffect(() => {
    if (loading || !data) {
      return;
    }

    setNowPlaying({
      album: data.currentTrack?.album,
      artist: data.currentTrack?.artist,
      title: data.currentTrack?.title,
      cover: data.currentTrack?.albumArt
        ? `http://localhost:6062/covers/${data.currentTrack?.albumArt}`
        : "",
      duration: data.currentTrack?.length || 0,
      progress: data.currentTrack?.elapsed || 0,
      isPlaying: playback?.status === 1,
      albumId: data.currentTrack?.albumId,
    });
  }, [data, loading, playback]);
  return (
    <ControlBar
      nowPlaying={nowPlaying}
      onPlay={() => resume()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
    />
  );
};

export default ControlBarWithData;
