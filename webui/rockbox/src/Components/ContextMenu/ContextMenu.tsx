/* eslint-disable @typescript-eslint/no-explicit-any */
import Add from "../Icons/Add";
import Heart from "../Icons/Heart";
import HeartOutline from "../Icons/HeartOutline";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { StatefulPopover } from "baseui/popover";
import { NestedMenus, StatefulMenu } from "baseui/menu";
import TrackIcon from "../Icons/Track";
import { useTheme } from "@emotion/react";
import ChildMenu from "./ChildMenu";
import { FC, useState } from "react";
import {
  AlbumCover,
  AlbumCoverAlt,
  Artist,
  Container,
  Hover,
  Icon,
  Separator,
  Title,
  Track,
  TrackInfos,
} from "./styles";

export type ContextMenuProps = {
  liked?: boolean;
  track: any;
  onPlayNext: (id: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onPlayLast: (id: string) => void;
  onAddShuffled: (id: string) => void;
  recentPlaylists: any[];
};

const ContextMenu: FC<ContextMenuProps> = ({
  liked = false,
  track,
  onPlayNext,
  // onCreatePlaylist,
  onPlayLast,
  onAddTrackToPlaylist,
  onAddShuffled,
  recentPlaylists,
}) => {
  const theme = useTheme();
  const [, setIsNewPlaylistModalOpen] = useState(false);
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
                {track.cover && <AlbumCover src={track.cover} />}
                {!track.cover && (
                  <AlbumCoverAlt>
                    <TrackIcon width={24} height={24} color="#a4a3a3" />
                  </AlbumCoverAlt>
                )}
                <TrackInfos>
                  <Title>{track.title}</Title>
                  <Artist>{track.artist}</Artist>
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
                                    onAddTrackToPlaylist(item.id, track.id);
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
                  ]}
                  onItemSelect={({ item }) => {
                    if (item.label === "Add to Playlist") {
                      return;
                    }
                    if (item.label === "Play Next") {
                      onPlayNext(track.id);
                    }
                    if (item.label === "Play Last") {
                      onPlayLast(track.id);
                    }
                    if (item.label === "Add Shuffled") {
                      onAddShuffled(track.id);
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
          }}
        >
          <Icon>
            <EllipsisHorizontal />
          </Icon>
        </StatefulPopover>
      </Hover>
      <Separator />
      <StatefulPopover
        autoFocus={false}
        placement="left"
        content={({ close }) => (
          <div style={{ width: 205 }}>
            <StatefulMenu
              overrides={{
                List: {
                  style: {
                    boxShadow: "none",
                    backgroundColor: theme.colors.popoverBackground,
                  },
                },
              }}
              items={[
                {
                  id: "1",
                  label: <div>Create new playlist</div>,
                },
              ]}
              onItemSelect={() => {
                setIsNewPlaylistModalOpen(true);
                close();
              }}
            />
          </div>
        )}
        overrides={{
          Inner: {
            style: {
              backgroundColor: theme.colors.popoverBackground,
            },
          },
        }}
      >
        <Icon>
          <Add size={24} color={theme.colors.icon} />
        </Icon>
      </StatefulPopover>
      <Separator />
      {liked && (
        <Icon>
          <Heart height={24} width={24} color={theme.colors.icon} />
        </Icon>
      )}
      {!liked && (
        <Icon>
          <HeartOutline height={24} width={24} color={theme.colors.icon} />
        </Icon>
      )}
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
