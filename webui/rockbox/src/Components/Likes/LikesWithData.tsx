import { FC, useEffect } from "react";
import Likes from "./Likes";
import { useGetLikedTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { useRecoilState } from "recoil";
import { likedTracks, likesState } from "./LikesState";

const LikesWithData: FC = () => {
  const [likes, setLikes] = useRecoilState(likesState);
  const { data, loading } = useGetLikedTracksQuery({
    fetchPolicy: "network-only",
  });
  const [tracks, setTracks] = useRecoilState(likedTracks);
  const { formatTime } = useTimeFormat();

  useEffect(() => {
    if (!data || loading) {
      return;
    }

    const updatedLikes: Record<string, boolean> = {
      ...likes,
    };
    data.likedTracks.forEach((x) => {
      updatedLikes[x.id!] = true;
    });
    setLikes(updatedLikes);

    setTracks(
      data.likedTracks.map((x, i) => ({
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

  const onPlayTrack = (trackId: string) => {
    console.log(">>", trackId);
  };

  return <Likes tracks={tracks} onPlayTrack={onPlayTrack} />;
};

export default LikesWithData;
