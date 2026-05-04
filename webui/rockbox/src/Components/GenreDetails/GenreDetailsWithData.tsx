import { FC } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  useGetGenreQuery,
  usePlayGenreTracksMutation,
} from "../../Hooks/useGenres";
import { useTimeFormat } from "../../Hooks/useFormat";
import GenreDetails from "./GenreDetails";

const GenreDetailsWithData: FC = () => {
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const { data, isLoading } = useGetGenreQuery({ id: id! });
  const { mutate: playGenreTracks } = usePlayGenreTracksMutation();
  const { formatTime } = useTimeFormat();

  const tracks =
    data?.genre?.tracks.map((t) => ({
      ...t,
      time: formatTime(t.length),
      albumArt: t.albumArt
        ? `${location.protocol}//${location.host}/covers/${t.albumArt}`
        : "",
    })) ?? [];

  const albums =
    data?.genre?.albums.map((a) => ({
      ...a,
      cover: a.albumArt
        ? `${location.protocol}//${location.host}/covers/${a.albumArt}`
        : "",
    })) ?? [];

  const artists = (data?.genre?.artists ?? []).map((a) => ({
    ...a,
    image: a.image
      ? a.image.startsWith("http")
        ? a.image
        : `${location.protocol}//${location.host}/covers/${a.image}`
      : null,
  }));

  const onPlayAll = () => playGenreTracks({ genreId: id!, shuffle: false });
  const onShuffleAll = () => playGenreTracks({ genreId: id!, shuffle: true });
  const onPlayTrack = (position: number) =>
    playGenreTracks({ genreId: id!, position });

  return (
    <GenreDetails
      id={data?.genre?.id ?? id ?? ""}
      name={data?.genre?.name ?? ""}
      trackCount={data?.genre?.trackCount ?? tracks.length}
      tracks={tracks}
      albums={albums}
      artists={artists}
      loading={isLoading}
      onPlayAll={onPlayAll}
      onShuffleAll={onShuffleAll}
      onPlayTrack={onPlayTrack}
      onGoBack={() => navigate(-1)}
    />
  );
};

export default GenreDetailsWithData;
