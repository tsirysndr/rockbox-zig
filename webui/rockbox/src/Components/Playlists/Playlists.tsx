/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC, useState } from "react";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Link } from "react-router-dom";
import Play from "../Icons/Play";
import { Music } from "@styled-icons/boxicons-regular";
import { Edit2, Trash2 } from "@styled-icons/feather";
import PlaylistModal from "./PlaylistModal";
import AlbumCardSkeleton from "../Skeletons/AlbumCardSkeleton";

export type PlaylistsProps = {
  savedPlaylists: any[];
  smartPlaylists: any[];
  loading?: boolean;
  onPlay: (id: string, isSmart: boolean) => void;
  onEdit: (playlist: any) => void;
  onDelete: (id: string) => void;
  onCreate: (name: string, description?: string) => Promise<void>;
  onUpdate: (id: string, name: string, description?: string) => Promise<void>;
};

const Playlists: FC<PlaylistsProps> = ({
  savedPlaylists,
  smartPlaylists,
  loading,
  onPlay,
  onDelete,
  onCreate,
  onUpdate,
}) => {
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [editPlaylist, setEditPlaylist] = useState<any>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  return (
    <div className="flex flex-row w-full h-full">
      <Sidebar active="playlists" />
      <MainView>
        <ControlBar />
        <div className="h-[var(--content-area-height)] overflow-y-auto">
          <div className="flex items-center justify-between pr-5 pl-5 mt-6 mb-5">
            <div className="text-2xl font-[RockfordSansMedium]">Playlists</div>
            <button
              onClick={() => setCreateModalOpen(true)}
              style={{
                background: "#6F00FF",
                color: "#fff",
                border: "none",
                borderRadius: 20,
                padding: "8px 18px",
                fontSize: 13,
                cursor: "pointer",
                fontFamily: "RockfordSansMedium",
              }}
            >
              + New Playlist
            </button>
          </div>

          {loading && (
            <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-5 px-5 mb-10">
              {Array.from({ length: 8 }).map((_, i) => (
                <div key={i}>
                  <AlbumCardSkeleton />
                </div>
              ))}
            </div>
          )}

          {!loading && savedPlaylists.length > 0 && (
            <>
              <div className="text-base font-[RockfordSansMedium] px-5 mb-4 mt-6 text-[var(--theme-secondary-text)]">MY PLAYLISTS</div>
              <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-5 px-5 mb-10">
                {savedPlaylists.map((playlist) => (
                  <div
                    className="relative cursor-pointer [&:hover_.card-actions]:opacity-100"
                    key={playlist.id}
                  >
                    <Link className="no-underline" to={`/playlists/${playlist.id}`}>
                      <div
                        className="w-full aspect-square rounded-[6px] bg-[var(--theme-cover)] flex items-center justify-center overflow-hidden"
                        style={playlist.image ? { backgroundImage: `url(${playlist.image})`, backgroundSize: 'cover', backgroundPosition: 'center' } : {}}
                      >
                        {!playlist.image && <Music size={48} color="#bbb" />}
                      </div>
                    </Link>
                    <div className="card-actions absolute bottom-12 left-2 right-2 flex flex-row items-center justify-between opacity-0 transition-opacity duration-150 pb-[6px]">
                      <button
                        className="h-9 w-9 rounded-[18px] border-0 cursor-pointer flex items-center justify-center bg-[var(--theme-surface)] backdrop-blur-[4px] hover:bg-[var(--theme-hover)]"
                        onClick={() => onPlay(playlist.id, false)}
                      >
                        <Play small color="var(--theme-icon)" />
                      </button>
                      <div style={{ display: "flex", gap: 4 }}>
                        <button
                          className="h-9 w-9 rounded-[18px] border-0 cursor-pointer flex items-center justify-center bg-[var(--theme-surface)] backdrop-blur-[4px] hover:bg-[var(--theme-hover)]"
                          onClick={() => setEditPlaylist(playlist)}
                        >
                          <Edit2 size={15} color="var(--theme-icon)" />
                        </button>
                        <button
                          className="h-9 w-9 rounded-[18px] border-0 cursor-pointer flex items-center justify-center bg-[var(--theme-surface)] backdrop-blur-[4px] hover:bg-[var(--theme-hover)]"
                          onClick={() => setDeleteConfirmId(playlist.id)}
                        >
                          <Trash2 size={15} color="#e00" />
                        </button>
                      </div>
                    </div>
                    <Link className="no-underline" to={`/playlists/${playlist.id}`}>
                      <div className="text-sm font-[RockfordSansMedium] mt-2 whitespace-nowrap overflow-hidden text-ellipsis text-[var(--theme-text)]">{playlist.name}</div>
                    </Link>
                    <div className="text-xs text-[var(--theme-secondary-text)] mt-[2px]">{playlist.trackCount} tracks</div>
                  </div>
                ))}
              </div>
            </>
          )}

          {!loading && smartPlaylists.length > 0 && (
            <>
              <div className="text-base font-[RockfordSansMedium] px-5 mb-4 mt-6 text-[var(--theme-secondary-text)]">SMART PLAYLISTS</div>
              <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-5 px-5 mb-10">
                {smartPlaylists.map((playlist) => (
                  <div
                    className="relative cursor-pointer [&:hover_.card-actions]:opacity-100"
                    key={playlist.id}
                  >
                    <Link className="no-underline" to={`/playlists/smart/${playlist.id}`}>
                      <div className="w-full aspect-square rounded-[6px] bg-[var(--theme-cover)] flex items-center justify-center overflow-hidden">
                        <Music size={48} color="#6F00FF" />
                      </div>
                    </Link>
                    <div className="card-actions absolute bottom-12 left-2 right-2 flex flex-row items-center justify-between opacity-0 transition-opacity duration-150 pb-[6px]">
                      <button
                        className="h-9 w-9 rounded-[18px] border-0 cursor-pointer flex items-center justify-center bg-[var(--theme-surface)] backdrop-blur-[4px] hover:bg-[var(--theme-hover)]"
                        onClick={() => onPlay(playlist.id, true)}
                      >
                        <Play small color="var(--theme-icon)" />
                      </button>
                    </div>
                    <Link className="no-underline" to={`/playlists/smart/${playlist.id}`}>
                      <div className="text-sm font-[RockfordSansMedium] mt-2 whitespace-nowrap overflow-hidden text-ellipsis text-[var(--theme-text)]">{playlist.name}</div>
                    </Link>
                    {playlist.description && (
                      <div className="text-xs text-[var(--theme-secondary-text)] mt-[2px]">{playlist.description}</div>
                    )}
                  </div>
                ))}
              </div>
            </>
          )}

          {!loading && savedPlaylists.length === 0 && smartPlaylists.length === 0 && (
            <div style={{ padding: "40px 20px", color: "#888", fontSize: 14 }}>
              No playlists yet. Create one to get started.
            </div>
          )}
        </div>
      </MainView>

      {createModalOpen && (
        <PlaylistModal
          title="New Playlist"
          onClose={() => setCreateModalOpen(false)}
          onSave={async (name, description) => {
            await onCreate(name, description);
            setCreateModalOpen(false);
          }}
        />
      )}

      {editPlaylist && (
        <PlaylistModal
          title="Edit Playlist"
          initialName={editPlaylist.name}
          initialDescription={editPlaylist.description}
          onClose={() => setEditPlaylist(null)}
          onSave={async (name, description) => {
            await onUpdate(editPlaylist.id, name, description);
            setEditPlaylist(null);
          }}
        />
      )}

      {deleteConfirmId && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
          onClick={() => setDeleteConfirmId(null)}
        >
          <div
            style={{
              background: "var(--theme-surface)",
              borderRadius: 12,
              padding: 28,
              width: 320,
              boxShadow: "0 8px 32px rgba(0,0,0,0.4)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div
              style={{
                fontSize: 16,
                fontFamily: "RockfordSansMedium",
                marginBottom: 12,
                color: "var(--theme-text)",
              }}
            >
              Delete playlist?
            </div>
            <div style={{ fontSize: 14, color: "var(--theme-secondary-text)", marginBottom: 24 }}>
              This action cannot be undone.
            </div>
            <div
              style={{
                display: "flex",
                gap: 10,
                justifyContent: "flex-end",
              }}
            >
              <button
                onClick={() => setDeleteConfirmId(null)}
                style={{
                  border: `1px solid ${"var(--theme-separator)"}`,
                  borderRadius: 8,
                  padding: "8px 16px",
                  cursor: "pointer",
                  background: "var(--theme-hover)",
                  color: "var(--theme-text)",
                  fontSize: 13,
                }}
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  onDelete(deleteConfirmId);
                  setDeleteConfirmId(null);
                }}
                style={{
                  background: "#e00",
                  color: "#fff",
                  border: "none",
                  borderRadius: 8,
                  padding: "8px 16px",
                  cursor: "pointer",
                  fontSize: 13,
                }}
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Playlists;
