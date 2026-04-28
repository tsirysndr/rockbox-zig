/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC, useState } from "react";
import { useTheme } from "@emotion/react";
import MainView from "../MainView";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  Container,
  Scrollable,
  Title,
  SectionTitle,
  PlaylistGrid,
  PlaylistCard,
  PlaylistCover,
  PlaylistName,
  PlaylistMeta,
  CardActions,
  CardAction,
  Link,
} from "./styles";
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
  const theme = useTheme();
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [editPlaylist, setEditPlaylist] = useState<any>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  return (
    <Container>
      <Sidebar active="playlists" />
      <MainView>
        <ControlBar />
        <Scrollable>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              paddingRight: 20,
            }}
          >
            <Title>Playlists</Title>
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
            <PlaylistGrid>
              {Array.from({ length: 8 }).map((_, i) => (
                <div key={i}>
                  <AlbumCardSkeleton />
                </div>
              ))}
            </PlaylistGrid>
          )}

          {!loading && savedPlaylists.length > 0 && (
            <>
              <SectionTitle>MY PLAYLISTS</SectionTitle>
              <PlaylistGrid>
                {savedPlaylists.map((playlist) => (
                  <PlaylistCard key={playlist.id}>
                    <Link to={`/playlists/${playlist.id}`}>
                      <PlaylistCover image={playlist.image}>
                        {!playlist.image && <Music size={48} color="#bbb" />}
                      </PlaylistCover>
                    </Link>
                    <CardActions className="card-actions">
                      <CardAction onClick={() => onPlay(playlist.id, false)}>
                        <Play small color={theme.colors.icon} />
                      </CardAction>
                      <div style={{ display: "flex", gap: 4 }}>
                        <CardAction onClick={() => setEditPlaylist(playlist)}>
                          <Edit2 size={15} color={theme.colors.icon} />
                        </CardAction>
                        <CardAction
                          onClick={() => setDeleteConfirmId(playlist.id)}
                        >
                          <Trash2 size={15} color="#e00" />
                        </CardAction>
                      </div>
                    </CardActions>
                    <Link to={`/playlists/${playlist.id}`}>
                      <PlaylistName>{playlist.name}</PlaylistName>
                    </Link>
                    <PlaylistMeta>{playlist.trackCount} tracks</PlaylistMeta>
                  </PlaylistCard>
                ))}
              </PlaylistGrid>
            </>
          )}

          {!loading && smartPlaylists.length > 0 && (
            <>
              <SectionTitle>SMART PLAYLISTS</SectionTitle>
              <PlaylistGrid>
                {smartPlaylists.map((playlist) => (
                  <PlaylistCard key={playlist.id}>
                    <Link to={`/playlists/smart/${playlist.id}`}>
                      <PlaylistCover>
                        <Music size={48} color="#6F00FF" />
                      </PlaylistCover>
                    </Link>
                    <CardActions className="card-actions">
                      <CardAction onClick={() => onPlay(playlist.id, true)}>
                        <Play small color={theme.colors.icon} />
                      </CardAction>
                    </CardActions>
                    <Link to={`/playlists/smart/${playlist.id}`}>
                      <PlaylistName>{playlist.name}</PlaylistName>
                    </Link>
                    {playlist.description && (
                      <PlaylistMeta>{playlist.description}</PlaylistMeta>
                    )}
                  </PlaylistCard>
                ))}
              </PlaylistGrid>
            </>
          )}

          {!loading && savedPlaylists.length === 0 && smartPlaylists.length === 0 && (
            <div style={{ padding: "40px 20px", color: "#888", fontSize: 14 }}>
              No playlists yet. Create one to get started.
            </div>
          )}
        </Scrollable>
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
              background: theme.colors.surface,
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
                color: theme.colors.text,
              }}
            >
              Delete playlist?
            </div>
            <div style={{ fontSize: 14, color: theme.colors.secondaryText, marginBottom: 24 }}>
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
                  border: `1px solid ${theme.colors.separator}`,
                  borderRadius: 8,
                  padding: "8px 16px",
                  cursor: "pointer",
                  background: theme.colors.hover,
                  color: theme.colors.text,
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
    </Container>
  );
};

export default Playlists;
