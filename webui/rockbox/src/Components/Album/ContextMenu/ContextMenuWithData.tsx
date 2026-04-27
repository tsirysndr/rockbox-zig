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
  const { mutate: playAlbum } = usePlayAlbumMutation();
  const { mutate: insertAlbum } = useInsertAlbumMutation();

  const onPlayShuffled = (albumId: string) => {
    playAlbum({ albumId, shuffle: true });
  };

  const onPlayNext = (albumId: string) => {
    insertAlbum({ albumId, position: PLAYLIST_INSERT_FIRST });
  };

  const onPlayLast = (albumId: string) => {
    insertAlbum({ albumId, position: PLAYLIST_INSERT_LAST });
  };

  const onAddShuffled = (albumId: string) => {
    insertAlbum({ albumId, position: PLAYLIST_INSERT_SHUFFLED });
  };

  const onPlayLastShuffled = (albumId: string) => {
    insertAlbum({ albumId, position: PLAYLIST_INSERT_LAST_SHUFFLED });
  };

  return (
    <ContextMenu
      album={item}
      onPlayNext={onPlayNext}
      onPlayLast={onPlayLast}
      onAddShuffled={onAddShuffled}
      onPlayLastShuffled={onPlayLastShuffled}
      onPlayShuffled={onPlayShuffled}
      recentPlaylists={[]}
      onCreatePlaylist={() => {}}
      onAddAlbumToPlaylist={() => {}}
    />
  );
};

export default ContextMenuWithData;
