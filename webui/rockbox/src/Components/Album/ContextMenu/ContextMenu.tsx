/* eslint-disable @typescript-eslint/no-explicit-any */
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { StatefulPopover } from "baseui/popover";
import { NestedMenus, StatefulMenu } from "baseui/menu";
import TrackIcon from "../../Icons/Track";
import ChildMenu from "./ChildMenu";
import { FC, useState } from "react";
import PlaylistModal from "../../Playlists/PlaylistModal";

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
                {album.cover && (
                  <img src={album.cover} className="h-[43px] w-[43px]" />
                )}
                {!album.cover && (
                  <div className="h-[43px] w-[43px] bg-[var(--theme-cover)] flex justify-center items-center">
                    <TrackIcon width={24} height={24} color="#a4a3a3" />
                  </div>
                )}
                <div className="ml-[10px] overflow-hidden">
                  <div className="text-sm text-ellipsis whitespace-nowrap overflow-hidden max-w-[125px] text-[var(--theme-text)]">
                    {album.title}
                  </div>
                  <div className="text-[rgb(170,170,180)] text-xs text-ellipsis whitespace-nowrap overflow-hidden max-w-[125px]">
                    {album.artist}
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
                backgroundColor: "var(--theme-popover-background)",
              },
            },
            Body: {
              style: {
                zIndex: 1,
              },
            },
          }}
        >
          <div className="h-10 w-10 rounded-full flex justify-center items-center border-0 cursor-pointer bg-transparent hover:bg-[#434242b5]">
            <EllipsisHorizontal size={24} color="#fff" />
          </div>
        </StatefulPopover>
      </button>
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
    </div>
  );
};

export default ContextMenu;
