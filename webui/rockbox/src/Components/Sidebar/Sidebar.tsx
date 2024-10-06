import { FC } from "react";
import { SidebarContainer, MenuItem } from "./styles";
import { Disc } from "@styled-icons/boxicons-regular";
import { HardDrive } from "@styled-icons/feather";
import Artist from "../Icons/Artist";
import Track from "../Icons/Track";

export type SidebarProps = {
  active: string;
};

const Sidebar: FC<SidebarProps> = ({ active }) => {
  return (
    <SidebarContainer>
      <MenuItem color={active === "albums" ? "#fe099c" : "initial"}>
        <Disc
          size={20}
          style={{ marginRight: 6 }}
          color={active === "albums" ? "#fe099c" : "initial"}
        />
        <div>Albums</div>
      </MenuItem>
      <MenuItem color={active === "artists" ? "#fe099c" : "initial"}>
        <Artist
          width={20}
          height={20}
          color={active === "artists" ? "#fe099c" : "initial"}
        />
        <div style={{ marginLeft: 6 }}>Artists</div>
      </MenuItem>
      <MenuItem color={active === "songs" ? "#fe099c" : "initial"}>
        <Track height={20} color={active === "songs" ? "#fe099c" : "initial"} />
        <div style={{ marginLeft: 6 }}>Songs</div>
      </MenuItem>
      <MenuItem color={active === "files" ? "#fe099c" : "initial"}>
        <HardDrive
          size={19}
          style={{ marginRight: 6 }}
          color={active === "files" ? "#fe099c" : "initial"}
        />
        <div>Files</div>
      </MenuItem>
    </SidebarContainer>
  );
};

export default Sidebar;
