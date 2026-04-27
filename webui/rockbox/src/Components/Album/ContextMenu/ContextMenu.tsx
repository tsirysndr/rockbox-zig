/* eslint-disable @typescript-eslint/no-explicit-any */
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { StatefulPopover } from "baseui/popover";
import { NestedMenus, StatefulMenu } from "baseui/menu";
import TrackIcon from "../../Icons/Track";
import { useTheme } from "@emotion/react";
import ChildMenu from "./ChildMenu";
import { FC, useState } from "react";
import PlaylistModal from "../../Playlists/PlaylistModal";
import {
  AlbumCover,
  AlbumCoverAlt,
  Artist,
  Container,
  Hover,
  Icon,
  Title,
  Track,
  TrackInfos,
} from "./styles";

export type ContextMenuProps = {
  album: any;
  onPlayNext: (id: string) => void;
  onCreatePlaylist: (name: string, albumId: string, description?: string) => void;
  onAddAlbumToPlaylist: (playlistId: string, albumId: string) => void;
  onPlayLast: (id: string) => void;
  onPlayShuffled: (id: string) => void;
  onAddShuffled: (id: string) => void;
  onPlayLastShuffled: (id: string) => void;
  recentPlaylists: any[];
};

const ContextMenu: FC<ContextMenuProps> = ({
  album,
  onPlayNext,
  onCreatePlaylist,
  onPlayLast,
  onAddAlbumToPlaylist,
  onPlayShuffled,
  onAddShuffled,
  onPlayLastShuffled,
  recentPlaylists,
}) => {
  const theme = useTheme();
  const [isNewPlaylistModalOpen, setIsNewPlaylistModalOpen] = useState(false);
  return (
    <Container>
      <Hover>
        <StatefulPopover
          placement="left"
          autoFocus={false}
          content={({ close }) => (
            <div
              style={{
                width: 205,
              }}
            >
              <Track>
                {album.cover && <AlbumCover src={album.cover} />}
                {!album.cover && (
                  <AlbumCoverAlt>
                    <TrackIcon width={24} height={24} color="#a4a3a3" />
                  </AlbumCoverAlt>
                )}
                <TrackInfos>
                  <Title>{album.title}</Title>
                  <Artist>{album.artist}</Artist>
                </TrackInfos>
              </Track>
              <NestedMenus>
                <StatefulMenu
                  overrides={{
                    List: {
                      style: {
                        boxShadow: "none",
                        backgroundColor: theme.colors.popoverBackground,
                      },
                    },
                    Option: {
                      props: {
                        getChildMenu: (item: { label: string }) => {
                          if (item.label === "Add to Playlist") {
                            return (
                              <ChildMenu
                                recentPlaylists={recentPlaylists}
                                onSelect={(item: {
                                  id: string;
                                  label: string;
                                }) => {
                                  if (item.label === "Create new playlist") {
                                    setIsNewPlaylistModalOpen(true);
                                  } else {
                                    onAddAlbumToPlaylist(item.id, album.id);
                                  }
                                  close();
                                }}
                              />
                            );
                          }
                          return null;
                        },
                      },
                    },
                  }}
                  items={[
                    {
                      id: "1",
                      label: "Play Next",
                    },
                    {
                      id: "2",
                      label: "Add to Playlist",
                    },
                    {
                      id: "3",
                      label: "Play Last",
                    },
                    {
                      id: "4",
                      label: "Add Shuffled",
                    },
                    {
                      id: "5",
                      label: "Play Last Shuffled",
                    },
                    {
                      id: "6",
                      label: "Play Shuffled",
                    },
                  ]}
                  onItemSelect={({ item }) => {
                    switch (item.label) {
                      case "Play Next":
                        onPlayNext(album.id);
                        break;
                      case "Play Last":
                        onPlayLast(album.id);
                        break;
                      case "Add Shuffled":
                        onAddShuffled(album.id);
                        break;
                      case "Play Last Shuffled":
                        onPlayLastShuffled(album.id);
                        break;
                      case "Play Shuffled":
                        onPlayShuffled(album.id);
                        break;
                    }
                    close();
                  }}
                />
              </NestedMenus>
            </div>
          )}
          overrides={{
            Inner: {
              style: {
                backgroundColor: theme.colors.popoverBackground,
              },
            },
            Body: {
              style: {
                zIndex: 1,
              },
            },
          }}
        >
          <Icon>
            <EllipsisHorizontal size={24} color="#fff" />
          </Icon>
        </StatefulPopover>
      </Hover>
      {isNewPlaylistModalOpen && (
        <PlaylistModal
          title="New Playlist"
          onClose={() => setIsNewPlaylistModalOpen(false)}
          onSave={async (name, description) => {
            await onCreatePlaylist(name, album.id, description);
            setIsNewPlaylistModalOpen(false);
          }}
        />
      )}
    </Container>
  );
};

export default ContextMenu;
