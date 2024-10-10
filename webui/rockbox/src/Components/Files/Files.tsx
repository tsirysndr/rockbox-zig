/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Folder2, MusicNoteBeamed } from "@styled-icons/bootstrap";
import {
  BackButton,
  ButtonGroup,
  Container,
  ContentWrapper,
  Directory,
  Hover,
  IconButton,
  MainView,
  Title,
} from "./styles";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { File } from "../../Types/file";
import Table from "../VirtualizedTable";
import "./styles.css";
import ArrowBack from "../Icons/ArrowBack";

const columnHelper = createColumnHelper<File>();
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
        }}
      >
        {info.row.original.isDirectory && <Folder2 size={20} />}
        {!info.row.original.isDirectory && <MusicNoteBeamed size={20} />}
      </div>
    ),
  }),
  columnHelper.accessor("name", {
    header: "",
    cell: (info) => <Directory href="#">{info.getValue()}</Directory>,
  }),
  columnHelper.accessor("name", {
    header: "",
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    cell: (_info) => (
      <ButtonGroup style={{ justifyContent: "flex-end", alignItems: "center" }}>
        <IconButton>
          <Hover>
            <EllipsisHorizontal size={24} />
          </Hover>
        </IconButton>
      </ButtonGroup>
    ),
  }),
];

export type FilesProps = {
  files: File[];
};

const Files: FC<FilesProps> = (props) => {
  return (
    <Container>
      <Sidebar active="files" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          <BackButton onClick={() => {}}>
            <div style={{ marginTop: 2 }}>
              <ArrowBack color={"#000"} />
            </div>
          </BackButton>
          <Title>Files</Title>
          <Table columns={columns as any} tracks={props.files} />
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default Files;
