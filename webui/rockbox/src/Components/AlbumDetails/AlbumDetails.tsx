import { FC, useState } from "react";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import {
  Container,
  MainView,
  AlbumCover,
  ContentWrapper,
  AlbumTitle,
  Header,
  AlbumInfos,
  Artist,
  Tracks,
  Year,
  ButtonGroup,
  Separator,
  BackButton,
  IconButton,
  Hover,
  Label,
} from "./styles";
import Button from "../Button";
import ArrowBack from "../Icons/ArrowBack";
import HeartOutline from "../Icons/HeartOutline";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import Add from "../Icons/Add";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";

type Track = {
  id: string;
  trackNumber: string;
  title: string;
  artist: string;
  time: string;
};

const tracks = [
  {
    id: "1",
    trackNumber: "1",
    title: "Gasoline (Live)",
    artist: "Seether",
    time: "2:57",
  },
  {
    id: "2",
    trackNumber: "2",
    title: "Driven Under (Live)",
    artist: "Seether",
    time: "4:58",
  },
  {
    id: "3",
    trackNumber: "3",
    title: "Diseased (Live)",
    artist: "Seether",
    time: "3:46",
  },
  {
    id: "4",
    trackNumber: "4",
    title: "Truth (Live)",
    artist: "Seether",
    time: "5:15",
  },
  {
    id: "5",
    trackNumber: "5",
    title: "Immortality (Live)",
    artist: "Seether",
    time: "5:02",
  },
  {
    id: "6",
    trackNumber: "6",
    title: "Tied My Hands (Live)",
    artist: "Seether",
    time: "5:16",
  },
  {
    id: "7",
    trackNumber: "7",
    title: "Sympathetic (Live)",
    artist: "Seether",
    time: "4:12",
  },
  {
    id: "8",
    trackNumber: "8",
    title: "Fine Again (Live)",
    artist: "Seether",
    time: "5:05",
  },
  {
    id: "9",
    trackNumber: "9",
    title: "Broken (Live)",
    artist: "Seether",
    time: "4:17",
  },
  {
    id: "10",
    trackNumber: "10",
    title: "The Gift (Live)",
    artist: "Seether",
    time: "5:36",
  },
  {
    id: "11",
    trackNumber: "11",
    title: "Remedy (Live)",
    artist: "Seether",
    time: "3:40",
  },
  {
    id: "12",
    trackNumber: "12",
    title: "Plastci Man (Live)",
    artist: "Seether",
    time: "3:34",
  },
  {
    id: "13",
    trackNumber: "13",
    title: "The Gist (Alternate Mix)",
    artist: "Seether",
    time: "3:24",
  },
];

const columnHelper = createColumnHelper<Track>();
const columns = [
  columnHelper.accessor("trackNumber", {
    header: "#",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("title", {
    header: "Title",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("artist", {
    header: "Artist",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("time", {
    header: "Time",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("id", {
    header: "",
    cell: (_info) => (
      <ButtonGroup style={{ justifyContent: "flex-end", alignItems: "center" }}>
        <IconButton>
          <Hover>
            <EllipsisHorizontal size={24} />
          </Hover>
        </IconButton>
        <IconButton>
          <Add color="#000" size={24} />
        </IconButton>
        <IconButton>
          <HeartOutline color="#000" />
        </IconButton>
      </ButtonGroup>
    ),
  }),
];

const AlbumDetails: FC = () => {
  const [data, _setData] = useState<Track[]>(() => [...tracks]);

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Container>
      <Sidebar active="albums" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => {}}>
            <div style={{ marginTop: 2 }}>
              <ArrowBack color={"#000"} />
            </div>
          </BackButton>
          <Header>
            <AlbumCover src="https://resources.tidal.com/images/f6f5f0a6/dc95/4561/9ca6/6ba1e0f6a062/320x320.jpg" />
            <AlbumInfos>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  justifyContent: "center",
                  height: "calc(240px - 12px)",
                }}
              >
                <AlbumTitle>One Cold Night (Live)</AlbumTitle>
                <Artist>Seether</Artist>
                <Tracks>13 TRACKS</Tracks>
                <Year>2006</Year>
              </div>
              <ButtonGroup>
                <Button onClick={() => {}} kind="primary">
                  <Label>
                    <Play small color="#fff" />
                    <div style={{ marginLeft: 7 }}>Play</div>
                  </Label>
                </Button>
                <Separator />
                <Button onClick={() => {}} kind="secondary">
                  <Label>
                    <Shuffle color="#fe099c" />
                    <div style={{ marginLeft: 7 }}>Shuffle</div>
                  </Label>
                </Button>
              </ButtonGroup>
            </AlbumInfos>
          </Header>
          <table style={{ width: "100%", marginTop: 31 }}>
            <thead>
              {table.getHeaderGroups().map((headerGroup) => (
                <tr
                  key={headerGroup.id}
                  style={{ height: 36, color: "rgba(0, 0, 0, 0.54)" }}
                >
                  {headerGroup.headers.map((header) => (
                    <th key={header.id} style={{ textAlign: "left" }}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </th>
                  ))}
                </tr>
              ))}
            </thead>
            <tbody>
              {table.getRowModel().rows.map((row) => (
                <tr key={row.id} style={{ height: 48 }}>
                  {row.getVisibleCells().map((cell) => (
                    <td key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default AlbumDetails;
