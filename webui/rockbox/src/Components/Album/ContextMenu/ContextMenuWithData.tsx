import { FC } from "react";
import ContextMenu from "./ContextMenu";
import {
  useInsertAlbumMutation,
  usePlayAlbumMutation,
} from "../../../Hooks/GraphQL";
import {
  PLAYLIST_INSERT_FIRST,
  PLAYLIST_INSERT_LAST,
  PLAYLIST_INSERT_LAST_SHUFFLED,
  PLAYLIST_INSERT_SHUFFLED,
} from "../../../Types/playlist";

export type ContextMenuWithDataProps = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  item: any;
};

const ContextMenuWithData: FC<ContextMenuWithDataProps> = ({ item }) => {
  const [playAlbum] = usePlayAlbumMutation();
  const [insertAlbum] = useInsertAlbumMutation();

  const onPlayShuffled = (albumId: string) => {
    playAlbum({
      variables: {
        albumId,
        shuffle: true,
      },
    });
  };

  const onPlayNext = (albumId: string) => {
    insertAlbum({
      variables: {
        albumId,
        position: PLAYLIST_INSERT_FIRST,
      },
    });
  };

  const onPlayLast = (albumId: string) => {
    insertAlbum({
      variables: {
        albumId,
        position: PLAYLIST_INSERT_LAST,
      },
    });
  };

  const onAddShuffled = (albumId: string) => {
    insertAlbum({
      variables: {
        albumId,
        position: PLAYLIST_INSERT_SHUFFLED,
      },
    });
  };

  const onPlayLastShuffled = (albumId: string) => {
    insertAlbum({
      variables: {
        albumId,
        position: PLAYLIST_INSERT_LAST_SHUFFLED,
      },
    });
  };

  return (
    <ContextMenu
      album={item}
      onPlayNext={onPlayNext}
      onCreatePlaylist={() => {}}
      onAddTrackToPlaylist={() => {}}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      onPlayLastShuffled={onPlayLastShuffled}
      onPlayShuffled={onPlayShuffled}
      recentPlaylists={[]}
    />
  );
};

export default ContextMenuWithData;
