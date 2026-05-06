import { FC } from "react";
import {
  Disc,
  Music,
  Home as HomeIcon,
  Category,
} from "@styled-icons/boxicons-regular";
import { HardDrive } from "@styled-icons/feather";
import Artist from "../Icons/Artist";
import Track from "../Icons/Track";
import RockboxLogo from "../../Assets/rockbox-icon.svg";
import HeartOutline from "../Icons/HeartOutline";
import { Options } from "@styled-icons/fluentui-system-regular";
import { Link } from "react-router-dom";

export type SidebarProps = {
  active: string;
  cover?: string;
};

const Sidebar: FC<SidebarProps> = ({ active, cover }) => {
  return (
    <div className={`flex flex-col h-screen w-[222px] p-5 ${cover ? 'bg-background' : 'bg-surface'}`}>
      <div className="flex flex-row items-center justify-between">
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
          <button className="flex bg-transparent border-0 cursor-pointer mt-[3px] h-16">
            <Options size={24} color="var(--theme-icon)" />
          </button>
        </Link>
      </div>
      <Link to="/" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "home" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <HomeIcon
          size={20}
          style={{ marginRight: 6 }}
          color={active === "home" ? "var(--theme-text)" : "var(--theme-icon)"}
        />
        <div>Home</div>
      </Link>
      <Link to="/albums" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "albums" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <Disc
          size={20}
          style={{ marginRight: 6 }}
          color={active === "albums" ? "var(--theme-text)" : "var(--theme-icon)"}
        />
        <div>Albums</div>
      </Link>
      <Link to="/artists" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "artists" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <Artist
          width={20}
          height={20}
          color={active === "artists" ? "var(--theme-text)" : "var(--theme-icon)"}
        />
        <div style={{ marginLeft: 6 }}>Artists</div>
      </Link>
      <Link to="/genres" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "genres" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <Category
          size={20}
          style={{ marginRight: 6 }}
          color={active === "genres" ? "var(--theme-text)" : "var(--theme-icon)"}
        />
        <div>Genres</div>
      </Link>
      <Link to="/tracks" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "songs" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <Track height={20} color={active === "songs" ? "var(--theme-text)" : "var(--theme-icon)"} />
        <div style={{ marginLeft: 6 }}>Songs</div>
      </Link>
      <Link to="/likes" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "likes" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <HeartOutline
          height={20}
          width={20}
          color={active === "likes" ? "var(--theme-text)" : "var(--theme-icon)"}
        />
        <div style={{ marginLeft: 6 }}>Likes</div>
      </Link>
      <Link to="/files" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "files" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <HardDrive
          size={19}
          style={{ marginRight: 6 }}
          color={active === "files" ? "var(--theme-text)" : "var(--theme-icon)"}
        />
        <div>Files</div>
      </Link>
      <Link to="/playlists" className={`flex items-center justify-start flex-row p-[10px] cursor-pointer text-sm no-underline rounded-lg ${active === "playlists" ? 'bg-hover text-text' : 'text-icon'} hover:bg-hover hover:text-text`}>
        <Music
          size={20}
          style={{ marginRight: 6 }}
          color={active === "playlists" ? "var(--theme-text)" : "var(--theme-icon)"}
        />
        <div>Playlists</div>
      </Link>
    </div>
  );
};

export default Sidebar;
