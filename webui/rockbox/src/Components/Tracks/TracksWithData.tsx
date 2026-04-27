import { FC, useEffect, useState } from "react";
import { useRecoilValue } from "recoil";
import { usePlayAllTracksMutation, useTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { Track } from "../../Types/track";
import { filterState } from "../Filter/FilterState";
import Tracks from "./Tracks";

const TracksWithData: FC = () => {
  const filter = useRecoilValue(filterState);
  const { data, isLoading } = useTracksQuery();
  const [tracks, setTracks] = useState<Track[]>([]);
  const { formatTime } = useTimeFormat();
  const { mutate: playAllTracks } = usePlayAllTracksMutation();

  useEffect(() => {
    if (filter.term.length > 0 && filter.results) {
      setTracks(
        filter.results.tracks.map((x, i) => ({
          id: x.id!,
          trackNumber: i + 1,
          title: x.title,
          artist: x.artist,
          album: x.album,
          time: formatTime(x.length),
          albumArt: x.albumArt
            ? `${location.protocol}//${location.host}/covers/${x.albumArt}`
            : undefined,
          albumId: x.albumId,
          artistId: x.artistId,
          path: x.path,
        }))
      );
      return;
    }
    if (data) {
      setTracks(
        data.tracks.map((x, i) => ({
          id: x.id!,
          trackNumber: i + 1,
          title: x.title,
          artist: x.artist,
          album: x.album,
          time: formatTime(x.length),
          albumArt: x.albumArt
            ? `${location.protocol}//${location.host}/covers/${x.albumArt}`
            : undefined,
          albumId: x.albumId,
          artistId: x.artistId,
          path: x.path,
        }))
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filter, data]);

  useEffect(() => {
    if (!data || isLoading) {
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
          ? `${location.protocol}//${location.host}/covers/${x.albumArt}`
          : undefined,
        albumId: x.albumId,
        artistId: x.artistId,
        path: x.path,
      }))
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, isLoading]);

  const onPlayTrack = (position: number) => {
    playAllTracks({ position });
  };

  return (
    <Tracks
      tracks={tracks}
      onPlayTrack={onPlayTrack}
      keyword={filter.term}
      loading={isLoading}
    />
  );
};

export default TracksWithData;
