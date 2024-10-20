import { useTheme } from "@emotion/react";
import { StatefulMenu } from "baseui/menu";
import { FC } from "react";

const ChildMenu: FC<{
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  recentPlaylists: any[];
  onSelect: (item: { id: string; label: string }) => void;
}> = ({ onSelect, recentPlaylists }) => {
  const theme = useTheme();
  const items =
    recentPlaylists.length > 0
      ? {
          __ungrouped: [
            {
              label: "Create new playlist",
            },
          ],
          RECENT: recentPlaylists.map((playlist) => ({
            id: playlist.id,
            label: <div>{playlist.name}</div>,
          })),
        }
      : {
          __ungrouped: [
            {
              label: "Create new playlist",
            },
          ],
        };
  return (
    <StatefulMenu
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      items={items as any}
      overrides={{
        List: {
          style: {
            boxShadow: "none",
            backgroundColor: theme.colors.popoverBackground,
          },
        },
      }}
      onItemSelect={({ item }) => onSelect(item)}
    />
  );
};

export default ChildMenu;
