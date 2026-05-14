/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Folder2, HddNetwork, MusicNoteBeamed, PlayBtn } from "@styled-icons/bootstrap";
import { Link } from "react-router-dom";
import { File } from "../../Types/file";
import Table from "../Table";
import "./styles.css";
import ArrowBack from "../Icons/ArrowBack";
import MainView from "../MainView";
import ContextMenu from "./ContextMenu";
import Play from "../Icons/Play";
import FileListSkeleton from "../Skeletons/FileListSkeleton";
import JellyfinIcon from "../Icons/Jellyfin";
import NavidromeIcon from "../Icons/Navidrome";
import KodiIcon from "../Icons/Kodi";

const columnHelper = createColumnHelper<File>();

export type FilesProps = {
  files: File[];
  canGoBack: boolean;
  onGoBack: () => void;
  refetching?: boolean;
  onPlayTrack: (path: string, index: number) => void;
  onPlayDirectory: (path: string) => void;
  onNavigateDirectory?: (file: File) => void;
};

const Files: FC<FilesProps> = (props) => {
  const columns = [
    columnHelper.accessor("name", {
      header: "",
      size: 15,
      cell: (info) => (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            marginLeft: 10,
          }}
        >
          {info.row.original.isDirectory &&
            info.row.original.path !== "__local__" &&
            !info.row.original.path.startsWith("upnp://") &&
            !info.row.original.path.startsWith("plex://") &&
            !info.row.original.path.startsWith("jellyfin://") &&
            !info.row.original.path.startsWith("navidrome://") &&
            !info.row.original.path.startsWith("kodi://") && (
            <div>
              <div
                className="play"
                onClick={() => props.onPlayDirectory(info.row.original.path)}
              >
                <Play small color="var(--theme-icon)" />
              </div>
              <div className="folder">
                <Folder2 size={20} />
              </div>
            </div>
          )}
          {info.row.original.path === "__local__" && (
            <div className="no-play">
              <div className="folder">
                <Folder2 size={20} />
              </div>
            </div>
          )}
          {info.row.original.path === "upnp://" && (
            <div className="no-play">
              <div className="folder">
                <HddNetwork size={20} />
              </div>
            </div>
          )}
          {info.row.original.path === "plex://" && (
            <div className="no-play">
              <div className="folder">
                <PlayBtn size={20} />
              </div>
            </div>
          )}
          {info.row.original.isDirectory &&
            info.row.original.path.startsWith("plex://") &&
            info.row.original.path !== "plex://" && (
            <div className="no-play">
              <div className="folder">
                <Folder2 size={20} />
              </div>
            </div>
          )}
          {info.row.original.path === "jellyfin://" && (
            <div className="no-play">
              <div className="folder">
                <JellyfinIcon size={20} color="currentColor" />
              </div>
            </div>
          )}
          {info.row.original.isDirectory &&
            info.row.original.path.startsWith("jellyfin://") &&
            info.row.original.path !== "jellyfin://" && (
            <div className="no-play">
              <div className="folder">
                <Folder2 size={20} />
              </div>
            </div>
          )}
          {info.row.original.path === "navidrome://" && (
            <div className="no-play">
              <div className="folder">
                <NavidromeIcon size={20} color="currentColor" />
              </div>
            </div>
          )}
          {info.row.original.isDirectory &&
            info.row.original.path.startsWith("navidrome://") &&
            info.row.original.path !== "navidrome://" && (
            <div className="no-play">
              <div className="folder">
                <Folder2 size={20} />
              </div>
            </div>
          )}
          {info.row.original.path === "kodi://" && (
            <div className="no-play">
              <div className="folder">
                <KodiIcon size={20} color="currentColor" />
              </div>
            </div>
          )}
          {info.row.original.isDirectory &&
            info.row.original.path.startsWith("kodi://") &&
            info.row.original.path !== "kodi://" && (
            <div className="no-play">
              <div className="folder">
                <Folder2 size={20} />
              </div>
            </div>
          )}
          {!info.row.original.isDirectory && (
            <div>
              <div
                className="play"
                onClick={() => {
                  const parent = info.row.original.path.split("/").slice(0, -1);
                  props.onPlayTrack(parent.join("/") || "/", info.row.index);
                }}
              >
                <Play small color="var(--theme-icon)" />
              </div>
              <div className="folder">
                <MusicNoteBeamed size={20} />
              </div>
            </div>
          )}
        </div>
      ),
    }),
    columnHelper.accessor("name", {
      header: "",
      cell: (info) => (
        <>
          {info.row.original.isDirectory && props.onNavigateDirectory && (
            <Link
              className="text-[var(--theme-text)] ml-[10px] no-underline font-[RockfordSansRegular] w-[calc(100vw-500px)] max-w-[calc(100vw-500px)] text-ellipsis overflow-hidden whitespace-nowrap block hover:underline"
              to="#"
              onClick={(e) => {
                e.preventDefault();
                props.onNavigateDirectory!(info.row.original);
              }}
            >
              {info.getValue()}
            </Link>
          )}
          {info.row.original.isDirectory && !props.onNavigateDirectory && (
            <Link
              className="text-[var(--theme-text)] ml-[10px] no-underline font-[RockfordSansRegular] w-[calc(100vw-500px)] max-w-[calc(100vw-500px)] text-ellipsis overflow-hidden whitespace-nowrap block hover:underline"
              to={`/files?q=${info.row.original.path}`}
            >
              {info.getValue()}
            </Link>
          )}
          {!info.row.original.isDirectory && (
            <div
              className="text-[var(--theme-text)] ml-[10px] no-underline font-[RockfordSansRegular] w-[calc(100vw-500px)] max-w-[calc(100vw-500px)] text-ellipsis overflow-hidden whitespace-nowrap block cursor-pointer hover:underline"
              onClick={() => {
                const parent = info.row.original.path.split("/").slice(0, -1);
                props.onPlayTrack(parent.join("/") || "/", info.row.index);
              }}
            >
              {info.getValue()}
            </div>
          )}
        </>
      ),
    }),
    columnHelper.accessor("name", {
      header: "",
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      cell: (info) => {
        const isRootEntry =
          info.row.original.path === "__local__" ||
          info.row.original.path.startsWith("upnp://") ||
          info.row.original.path.startsWith("plex://") ||
          info.row.original.path.startsWith("jellyfin://") ||
          info.row.original.path.startsWith("navidrome://") ||
          info.row.original.path.startsWith("kodi://");
        if (isRootEntry) return <div className="flex flex-row items-center" />;
        return (
          <div
            className="flex flex-row items-center"
            style={{ justifyContent: "flex-end", alignItems: "center" }}
          >
            <ContextMenu
              entry={{
                title: info.row.original.name,
                isDirectory: info.row.original.isDirectory,
                path: info.row.original.path,
              }}
            />
          </div>
        );
      },
    }),
  ];

  return (
    <div className="flex flex-row w-full h-full">
      <Sidebar active="files" />
      <MainView>
        <ControlBar />
        <div className="overflow-y-auto h-[calc(100vh-100px)] px-5">
          {props.canGoBack && (
            <button
              className="border-0 cursor-pointer flex items-center justify-center h-[30px] w-[30px] left-5 rounded-[15px] bg-[var(--theme-back-button)] mt-[45px] mb-[46px] absolute z-[1]"
              onClick={() => props.onGoBack()}
            >
              <div style={{ marginTop: 2 }}>
                <ArrowBack color="var(--theme-icon)" />
              </div>
            </button>
          )}
          <div className="text-2xl font-[RockfordSansMedium] mb-5">Files</div>
          {!props.refetching && (
            <Table columns={columns as any} tracks={props.files as any} />
          )}
          {props.refetching && <FileListSkeleton />}
        </div>
      </MainView>
    </div>
  );
};

export default Files;
