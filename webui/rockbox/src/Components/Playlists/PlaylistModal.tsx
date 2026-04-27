import { FC, useState } from "react";

type PlaylistModalProps = {
  title: string;
  initialName?: string;
  initialDescription?: string;
  onClose: () => void;
  onSave: (name: string, description?: string) => Promise<void>;
};

const PlaylistModal: FC<PlaylistModalProps> = ({
  title,
  initialName = "",
  initialDescription = "",
  onClose,
  onSave,
}) => {
  const [name, setName] = useState(initialName);
  const [description, setDescription] = useState(initialDescription);
  const [saving, setSaving] = useState(false);

  async function handleSave() {
    if (!name.trim()) return;
    setSaving(true);
    await onSave(name.trim(), description.trim() || undefined);
    setSaving(false);
  }

  return (
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
      onClick={onClose}
    >
      <div
        style={{
          background: "#fff",
          borderRadius: 12,
          padding: 28,
          width: 380,
          boxShadow: "0 8px 32px rgba(0,0,0,0.15)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div
          style={{
            fontSize: 18,
            fontFamily: "RockfordSansMedium",
            marginBottom: 20,
          }}
        >
          {title}
        </div>
        <div style={{ marginBottom: 14 }}>
          <label
            style={{
              fontSize: 12,
              color: "#555",
              display: "block",
              marginBottom: 4,
            }}
          >
            Name
          </label>
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSave()}
            style={{
              width: "100%",
              border: "1px solid #ddd",
              borderRadius: 8,
              padding: "8px 10px",
              fontSize: 14,
              outline: "none",
              boxSizing: "border-box",
            }}
          />
        </div>
        <div style={{ marginBottom: 24 }}>
          <label
            style={{
              fontSize: 12,
              color: "#555",
              display: "block",
              marginBottom: 4,
            }}
          >
            Description (optional)
          </label>
          <input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            style={{
              width: "100%",
              border: "1px solid #ddd",
              borderRadius: 8,
              padding: "8px 10px",
              fontSize: 14,
              outline: "none",
              boxSizing: "border-box",
            }}
          />
        </div>
        <div style={{ display: "flex", gap: 10, justifyContent: "flex-end" }}>
          <button
            onClick={onClose}
            style={{
              border: "1px solid #ddd",
              borderRadius: 8,
              padding: "8px 16px",
              cursor: "pointer",
              background: "#fff",
              fontSize: 13,
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={!name.trim() || saving}
            style={{
              background: name.trim() ? "#fe099c" : "#ccc",
              color: "#fff",
              border: "none",
              borderRadius: 8,
              padding: "8px 16px",
              cursor: name.trim() ? "pointer" : "default",
              fontSize: 13,
            }}
          >
            {saving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
};

export default PlaylistModal;
