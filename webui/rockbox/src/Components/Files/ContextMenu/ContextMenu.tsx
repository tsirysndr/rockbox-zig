/* eslint-disable @typescript-eslint/no-explicit-any */
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { Folder2 } from "@styled-icons/bootstrap";
import { StatefulPopover } from "baseui/popover";
import { NestedMenus, StatefulMenu } from "baseui/menu";
import TrackIcon from "../../Icons/Track";
import { useTheme } from "@emotion/react";
import ChildMenu from "./ChildMenu";
import { FC, useMemo, useState } from "react";
import {
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
  entry: any;
  onPlayNext: (id: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onPlayLast: (id: string) => void;
  onAddShuffled: (id: string) => void;
  onPlayShuffled: (id: string) => void;
  recentPlaylists: any[];
};

const ContextMenu: FC<ContextMenuProps> = ({
  entry,
  onPlayNext,
  // onCreatePlaylist,
  onPlayLast,
  onAddTrackToPlaylist,
  onAddShuffled,
  onPlayShuffled,
  recentPlaylists,
}) => {
  const theme = useTheme();
  const [, setIsNewPlaylistModalOpen] = useState(false);
  const items = useMemo(() => {
    if (entry.isDirectory) {
      return [
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
      ];
    }
    return [
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
    ];
  }, [entry]);
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
                {entry.isDirectory && (
                  <AlbumCoverAlt>
                    <Folder2 size={18} />
                  </AlbumCoverAlt>
                )}
                {!entry.isDirectory && (
                  <AlbumCoverAlt>
                    <TrackIcon width={24} height={24} color="#a4a3a3" />
                  </AlbumCoverAlt>
                )}
                <TrackInfos>
                  <Title>{entry.title}</Title>
                  <Artist>{entry.artist}</Artist>
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
                                    onAddTrackToPlaylist(item.id, entry.id);
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
                  items={items}
                  onItemSelect={({ item }) => {
                    if (item.label === "Add to Playlist") {
                      return;
                    }
                    if (item.label === "Play Next") {
                      onPlayNext(entry.id);
                    }
                    if (item.label === "Play Last") {
                      onPlayLast(entry.id);
                    }
                    if (item.label === "Add Shuffled") {
                      onAddShuffled(entry.id);
                    }
                    if (item.label === "Play Shuffled") {
                      onPlayShuffled(entry.id);
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
            <EllipsisHorizontal size={24} />
          </Icon>
        </StatefulPopover>
      </Hover>
      {/*<NewPlaylistModal
        onClose={() => {
          setIsNewPlaylistModalOpen(false);
        }}
        isOpen={isNewPlaylistModalOpen}
        onCreatePlaylist={onCreatePlaylist}
      />
      */}
    </Container>
  );
};

export default ContextMenu;
