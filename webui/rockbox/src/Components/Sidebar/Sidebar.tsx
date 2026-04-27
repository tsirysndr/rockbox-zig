import { FC } from "react";
import { SidebarContainer, MenuItem, Header, SettingsButton } from "./styles";
import { Disc, Music } from "@styled-icons/boxicons-regular";
import { HardDrive } from "@styled-icons/feather";
import Artist from "../Icons/Artist";
import Track from "../Icons/Track";
import RockboxLogo from "../../Assets/rockbox-icon.svg";
import HeartOutline from "../Icons/HeartOutline";
import { Options } from "@styled-icons/fluentui-system-regular";
import { Link } from "react-router-dom";
import { useTheme } from "@emotion/react";

export type SidebarProps = {
  active: string;
  cover?: string;
};

const Sidebar: FC<SidebarProps> = ({ active, cover }) => {
  const theme = useTheme();
  const icon = theme.colors.icon;
  const activeColor = theme.colors.text;

  return (
    <SidebarContainer cover={cover}>
      <Header>
        <a href="/" style={{ textDecoration: "none" }}>
          <img
            src={RockboxLogo}
            alt="Rockbox"
            style={{
              width: 40,
              marginBottom: 20,
              marginLeft: 12,
              marginTop: -12,
            }}
          />
        </a>
        <Link to="/settings">
          <SettingsButton>
            <Options size={24} color={icon} />
          </SettingsButton>
        </Link>
      </Header>
      <MenuItem active={active === "albums"} to="/albums">
        <Disc
          size={20}
          style={{ marginRight: 6 }}
          color={active === "albums" ? activeColor : icon}
        />
        <div>Albums</div>
      </MenuItem>
      <MenuItem active={active === "artists"} to="/artists">
        <Artist
          width={20}
          height={20}
          color={active === "artists" ? activeColor : icon}
        />
        <div style={{ marginLeft: 6 }}>Artists</div>
      </MenuItem>
      <MenuItem active={active === "songs"} to="/tracks">
        <Track height={20} color={active === "songs" ? activeColor : icon} />
        <div style={{ marginLeft: 6 }}>Songs</div>
      </MenuItem>
      <MenuItem active={active === "likes"} to="/likes">
        <HeartOutline
          height={20}
          width={20}
          color={active === "likes" ? activeColor : icon}
        />
        <div style={{ marginLeft: 6 }}>Likes</div>
      </MenuItem>
      <MenuItem active={active === "files"} to="/files">
        <HardDrive
          size={19}
          style={{ marginRight: 6 }}
          color={active === "files" ? activeColor : icon}
        />
        <div>Files</div>
      </MenuItem>
      <MenuItem active={active === "playlists"} to="/playlists">
        <Music
          size={20}
          style={{ marginRight: 6 }}
          color={active === "playlists" ? activeColor : icon}
        />
        <div>Playlists</div>
      </MenuItem>
    </SidebarContainer>
  );
};

export default Sidebar;
