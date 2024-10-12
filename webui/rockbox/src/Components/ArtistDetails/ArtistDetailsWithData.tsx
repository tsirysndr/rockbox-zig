import { FC } from "react";
import ArtistDetails from "./ArtistDetails";
import { useNavigate, useParams } from "react-router-dom";
import { useGetArtistQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";

const ArtistDetailsWithData: FC = () => {
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const { data } = useGetArtistQuery({
    variables: {
      id: id!,
    },
  });
  const { formatTime } = useTimeFormat();
  const tracks =
    data?.artist?.tracks.map((x) => ({
      ...x,
      time: formatTime(x.length),
      albumArt: x.albumArt ? `http://localhost:6062/covers/${x.albumArt}` : "",
    })) || [];
  const albums =
    data?.artist?.albums.map((x) => ({
      ...x,
      albumArt: x.albumArt ? `http://localhost:6062/covers/${x.albumArt}` : "",
    })) || [];
  return (
    <ArtistDetails
      name={data?.artist?.name || ""}
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      tracks={(tracks as any) || []}
      albums={albums || []}
      onPlayAll={() => {}}
      onShuffleAll={() => {}}
      onPlayAlbum={() => {}}
      onLikeAlbum={() => {}}
      onUnLikeAlbum={() => {}}
      onLikeTrack={() => {}}
      onUnlikeTrack={() => {}}
      onGoBack={() => navigate(-1)}
    />
  );
};

export default ArtistDetailsWithData;
