import { FC, useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useRecoilValue } from "recoil";
import { useGetAlbumQuery, usePlayAlbumMutation } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { Track } from "../../Types/track";
import { settingsState } from "../Settings/SettingsState";
import AlbumDetails from "./AlbumDetails";

const AlbumDetailsWithData: FC = () => {
  const { enableBlur } = useRecoilValue(settingsState);
  const [volumes, setVolumes] = useState<Track[][]>([]);
  const [tracks, setTracks] = useState<Track[]>([]);
  const { formatTime } = useTimeFormat();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const { data, refetch } = useGetAlbumQuery({
    variables: {
      id: id!,
    },
  });
  const [playAlbum] = usePlayAlbumMutation();

  const album = useMemo(
    () =>
      data
        ? {
            ...data.album!,
            albumArt: data.album?.albumArt
              ? `${location.protocol}//${location.host}/covers/${data.album?.albumArt}`
              : "",
          }
        : null,
    [data]
  );

  useEffect(() => {
    if (!album) {
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
        albumArt: album.albumArt,
        path: x.path,
      })) || []
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [album]);

  useEffect(() => {
    refetch();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  useEffect(() => {
    if (!tracks.length) {
      return;
    }
    getVolumes();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tracks]);

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

  function onPlayAll(shuffle: boolean) {
    playAlbum({
      variables: {
        albumId: id!,
        shuffle,
      },
    });
  }

  function onPlayTrack(position: number, disc: number) {
    let realPosition = 0;
    if (disc > 1) {
      // get the real position, since we have multiple discs,
      // we need to calculate the real position,
      // disc are ordered by volume
      for (let i = 0; i < disc - 1; i++) {
        realPosition += volumes[i].length;
      }
      realPosition += position;
    }
    playAlbum({
      variables: {
        albumId: id!,
        position: disc > 1 ? realPosition : position,
      },
    });
  }

  return (
    <AlbumDetails
      onGoBack={() => navigate(-1)}
      onPlayAll={() => onPlayAll(false)}
      onShuffleAll={() => onPlayAll(true)}
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      tracks={tracks as any[]}
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      album={album as any}
      volumes={volumes}
      enableBlur={enableBlur}
      onPlayTrack={onPlayTrack}
    />
  );
};

export default AlbumDetailsWithData;
