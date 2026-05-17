import { FC } from "react";
import { Link, useLocation } from "react-router-dom";
import {
  Disc,
  Music,
  Home as HomeIcon,
} from "@styled-icons/boxicons-regular";
import { Options } from "@styled-icons/fluentui-system-regular";
import Artist from "../Icons/Artist";

type Tab = {
  to: string;
  label: string;
  icon: (active: boolean) => React.ReactNode;
  matchPrefix?: boolean;
};

const TABS: Tab[] = [
  {
    to: "/",
    label: "Home",
    icon: (active) => (
      <HomeIcon size={24} color={active ? "#6F00FF" : "var(--theme-icon)"} />
    ),
  },
  {
    to: "/albums",
    label: "Albums",
    matchPrefix: true,
    icon: (active) => (
      <Disc size={24} color={active ? "#6F00FF" : "var(--theme-icon)"} />
    ),
  },
  {
    to: "/artists",
    label: "Artists",
    matchPrefix: true,
    icon: (active) => (
      <Artist
        width={24}
        height={24}
        color={active ? "#6F00FF" : "var(--theme-icon)"}
      />
    ),
  },
  {
    to: "/tracks",
    label: "Library",
    matchPrefix: false,
    icon: (active) => (
      <Music size={24} color={active ? "#6F00FF" : "var(--theme-icon)"} />
    ),
  },
  {
    to: "/settings",
    label: "Settings",
    matchPrefix: true,
    icon: (active) => (
      <Options size={24} color={active ? "#6F00FF" : "var(--theme-icon)"} />
    ),
  },
];

const BottomTabs: FC = () => {
  const { pathname } = useLocation();

  const isActive = (tab: Tab) => {
    if (tab.to === "/") return pathname === "/";
    return tab.matchPrefix ? pathname.startsWith(tab.to) : pathname === tab.to;
  };

  return (
    <nav className="md:hidden fixed bottom-0 left-0 right-0 z-30 flex flex-row h-[60px] bg-[var(--theme-surface)] border-t border-[var(--theme-separator)]">
      {TABS.map((tab) => {
        const active = isActive(tab);
        return (
          <Link
            key={tab.to}
            to={tab.to}
            className="flex flex-col items-center justify-center flex-1 py-1 no-underline gap-[3px]"
          >
            {tab.icon(active)}
            <span
              className="text-[9px] font-medium"
              style={{ color: active ? "#6F00FF" : "var(--theme-icon)" }}
            >
              {tab.label}
            </span>
          </Link>
        );
      })}
    </nav>
  );
};

export default BottomTabs;
