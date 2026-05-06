/* eslint-disable @typescript-eslint/no-explicit-any */
import Add from "../Icons/Add";
import Heart from "../Icons/Heart";
import HeartOutline from "../Icons/HeartOutline";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { StatefulPopover } from "baseui/popover";
import { NestedMenus, StatefulMenu } from "baseui/menu";
import TrackIcon from "../Icons/Track";
import ChildMenu from "./ChildMenu";
import { FC, useState } from "react";
import PlaylistModal from "../Playlists/PlaylistModal";

export type ContextMenuProps = {
  liked?: boolean;
  track: any;
  onPlayNext: (path: string) => void;
  onCreatePlaylist: (name: string, trackId: string, description?: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onPlayLast: (path: string) => void;
  onAddShuffled: (path: string) => void;
  onLike: (trackId: string) => void;
  onUnlike: (trackId: string) => void;
  recentPlaylists: any[];
};

const ContextMenu: FC<ContextMenuProps> = ({
  liked = false,
  track,
  onPlayNext,
  onCreatePlaylist,
  onPlayLast,
  onAddTrackToPlaylist,
  onAddShuffled,
  onLike,
  onUnlike,
  recentPlaylists,
}) => {
  const [isNewPlaylistModalOpen, setIsNewPlaylistModalOpen] = useState(false);
  return (
    <div className="flex flex-row h-[45px]">
      <button className="text-[var(--theme-icon)] bg-transparent border-0 cursor-pointer hover:text-[var(--theme-text)] focus:text-[var(--theme-text)]">
        <StatefulPopover
          placement="left"
          autoFocus={false}
          content={({ close }) => (
            <div
              style={{
                width: 205,
              }}
            >
              <div className="h-[54px] flex flex-row items-center px-[5px] border-b border-separator">
                {track.cover && (
                  <img className="h-[43px] w-[43px]" src={track.cover} />
                )}
                {!track.cover && (
                  <div className="h-[43px] w-[43px] bg-[var(--theme-cover)] flex justify-center items-center">
                    <TrackIcon width={24} height={24} color="#a4a3a3" />
                  </div>
                )}
                <div className="ml-[10px] overflow-hidden">
                  <div className="text-sm text-ellipsis whitespace-nowrap overflow-hidden max-w-[125px] text-[var(--theme-text)]">
                    {track.title}
                  </div>
                  <div className="text-[rgb(170,170,180)] text-xs text-ellipsis whitespace-nowrap overflow-hidden max-w-[125px]">
                    {track.artist}
                  </div>
                </div>
              </div>
              <NestedMenus>
                <StatefulMenu
                  overrides={{
                    List: {
                      style: {
                        boxShadow: "none",
                        backgroundColor: "var(--theme-popover-background)",
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
                      onPlayNext(track.path);
                    }
                    if (item.label === "Play Last") {
                      onPlayLast(track.path);
                    }
                    if (item.label === "Add Shuffled") {
                      onAddShuffled(track.path);
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
                backgroundColor: "var(--theme-popover-background)",
              },
            },
          }}
        >
          <div className="cursor-pointer flex h-[45px] w-6 justify-center items-center">
            <EllipsisHorizontal />
          </div>
        </StatefulPopover>
      </button>
      <div className="w-[10px]" />
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
                    backgroundColor: "var(--theme-popover-background)",
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
              backgroundColor: "var(--theme-popover-background)",
            },
          },
        }}
      >
        <div className="cursor-pointer flex h-[45px] w-6 justify-center items-center">
          <Add size={24} color="var(--theme-icon)" />
        </div>
      </StatefulPopover>
      <div className="w-[10px]" />
      {liked && (
        <div
          className="cursor-pointer flex h-[45px] w-6 justify-center items-center"
          onClick={() => onUnlike(track.id)}
        >
          <Heart height={24} width={24} color={"#6F00FF"} />
        </div>
      )}
      {!liked && (
        <div
          className="cursor-pointer flex h-[45px] w-6 justify-center items-center"
          onClick={() => {
            onLike(track.id);
          }}
        >
          <HeartOutline height={24} width={24} color="var(--theme-icon)" />
        </div>
      )}
      {isNewPlaylistModalOpen && (
        <PlaylistModal
          title="New Playlist"
          onClose={() => setIsNewPlaylistModalOpen(false)}
          onSave={async (name, description) => {
            await onCreatePlaylist(name, track.id, description);
            setIsNewPlaylistModalOpen(false);
          }}
        />
      )}
    </div>
  );
};

export default ContextMenu;
