/* eslint-disable @typescript-eslint/no-explicit-any */
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { Folder2 } from "@styled-icons/bootstrap";
import { StatefulPopover } from "baseui/popover";
import { NestedMenus, StatefulMenu } from "baseui/menu";
import TrackIcon from "../../Icons/Track";
import ChildMenu from "./ChildMenu";
import { FC, useMemo, useState } from "react";
import PlaylistModal from "../../Playlists/PlaylistModal";

export type ContextMenuProps = {
  entry: any;
  onPlayNext: (path: string) => void;
  onCreatePlaylist: (name: string, trackId: string, description?: string) => void;
  onAddTrackToPlaylist: (playlistId: string, path: string) => void;
  onPlayLast: (path: string) => void;
  onAddShuffled: (path: string) => void;
  onPlayShuffled: (path: string) => void;
  onPlayLastShuffled: (path: string) => void;
  recentPlaylists: any[];
};

const ContextMenu: FC<ContextMenuProps> = ({
  entry,
  onPlayNext,
  onCreatePlaylist,
  onPlayLast,
  onAddTrackToPlaylist,
  onAddShuffled,
  onPlayShuffled,
  onPlayLastShuffled,
  recentPlaylists,
}) => {
  const [isNewPlaylistModalOpen, setIsNewPlaylistModalOpen] = useState(false);
  const items = useMemo(() => {
    const base = [
      { id: "1", label: "Play Next" },
      { id: "3", label: "Play Last" },
      { id: "4", label: "Add Shuffled" },
    ];
    if (entry.isDirectory) {
      return [
        ...base,
        { id: "5", label: "Play Last Shuffled" },
        { id: "6", label: "Play Shuffled" },
      ];
    }
    return [
      { id: "1", label: "Play Next" },
      { id: "2", label: "Add to Playlist" },
      { id: "3", label: "Play Last" },
      { id: "4", label: "Add Shuffled" },
    ];
  }, [entry]);

  return (
    <div className="flex flex-row h-[45px]">
      <button className="text-icon bg-transparent border-0 cursor-pointer hover:text-text focus:text-text">
        <StatefulPopover
          placement="left"
          autoFocus={false}
          content={({ close }) => (
            <div style={{ width: 205 }}>
              <div className="h-[54px] flex flex-row items-center px-[5px] border-b border-separator">
                {entry.isDirectory ? (
                  <div className="h-[43px] w-[43px] bg-cover flex justify-center items-center">
                    <Folder2 size={18} />
                  </div>
                ) : (
                  <div className="h-[43px] w-[43px] bg-cover flex justify-center items-center">
                    <TrackIcon width={24} height={24} color="#a4a3a3" />
                  </div>
                )}
                <div className="ml-[10px] overflow-hidden">
                  <div className="text-sm text-ellipsis whitespace-nowrap overflow-hidden max-w-[125px] text-text">{entry.title}</div>
                  <div className="text-[rgb(170,170,180)] text-xs text-ellipsis whitespace-nowrap overflow-hidden max-w-[125px]">{entry.artist}</div>
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
                                onSelect={(item: { id: string; label: string }) => {
                                  if (item.label === "Create new playlist") {
                                    setIsNewPlaylistModalOpen(true);
                                  } else {
                                    onAddTrackToPlaylist(item.id, entry.path);
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
                    switch (item.label) {
                      case "Play Next":
                        onPlayNext(entry.path);
                        break;
                      case "Play Last":
                        onPlayLast(entry.path);
                        break;
                      case "Add Shuffled":
                        onAddShuffled(entry.path);
                        break;
                      case "Play Shuffled":
                        onPlayShuffled(entry.path);
                        break;
                      case "Play Last Shuffled":
                        onPlayLastShuffled(entry.path);
                        break;
                      case "Add to Playlist":
                        return;
                      default:
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
              style: { backgroundColor: "var(--theme-popover-background)" },
            },
            Body: { style: { zIndex: 1 } },
          }}
        >
          <div className="cursor-pointer flex h-[45px] w-6 justify-center items-center">
            <EllipsisHorizontal size={24} />
          </div>
        </StatefulPopover>
      </button>
      {isNewPlaylistModalOpen && (
        <PlaylistModal
          title="New Playlist"
          onClose={() => setIsNewPlaylistModalOpen(false)}
          onSave={async (name, description) => {
            await onCreatePlaylist(name, entry.path, description);
            setIsNewPlaylistModalOpen(false);
          }}
        />
      )}
    </div>
  );
};

export default ContextMenu;
