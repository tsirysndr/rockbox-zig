import { FC } from "react";
import { SidebarContainer, MenuItem } from "./styles";
import { Disc } from "@styled-icons/boxicons-regular";
import { HardDrive } from "@styled-icons/feather";
import Artist from "../Icons/Artist";
import Track from "../Icons/Track";
import RockboxLogo from "../../Assets/rockbox-icon.svg";

export type SidebarProps = {
  active: string;
  cover?: string;
};

const Sidebar: FC<SidebarProps> = ({ active, cover }) => {
  return (
    <SidebarContainer cover={cover}>
      <a href="/" style={{ textDecoration: "none" }}>
        <img
          src={RockboxLogo}
          alt="Rockbox"
          style={{ width: 40, marginBottom: 20, marginLeft: 12 }}
        />
      </a>
      <MenuItem
        color={active === "albums" ? "#fe099c" : "initial"}
        to="/albums"
      >
        <Disc
          size={20}
          style={{ marginRight: 6 }}
          color={active === "albums" ? "#fe099c" : "initial"}
        />
        <div>Albums</div>
      </MenuItem>
      <MenuItem
        color={active === "artists" ? "#fe099c" : "initial"}
        to="/artists"
      >
        <Artist
          width={20}
          height={20}
          color={active === "artists" ? "#fe099c" : "initial"}
        />
        <div style={{ marginLeft: 6 }}>Artists</div>
      </MenuItem>
      <MenuItem color={active === "songs" ? "#fe099c" : "initial"} to="/tracks">
        <Track height={20} color={active === "songs" ? "#fe099c" : "initial"} />
        <div style={{ marginLeft: 6 }}>Songs</div>
      </MenuItem>
      <MenuItem color={active === "files" ? "#fe099c" : "initial"} to="/files">
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
