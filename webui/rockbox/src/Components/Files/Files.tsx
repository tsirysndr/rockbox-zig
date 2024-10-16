/* eslint-disable @typescript-eslint/no-explicit-any */
import { FC } from "react";
import { createColumnHelper } from "@tanstack/react-table";
import Sidebar from "../Sidebar";
import ControlBar from "../ControlBar";
import { Folder2, MusicNoteBeamed } from "@styled-icons/bootstrap";
import {
  AudioFile,
  BackButton,
  ButtonGroup,
  Container,
  ContentWrapper,
  Directory,
  Hover,
  IconButton,
  Title,
} from "./styles";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { File } from "../../Types/file";
import Table from "../Table";
import "./styles.css";
import ArrowBack from "../Icons/ArrowBack";
import { Spinner } from "baseui/spinner";
import MainView from "../MainView";

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
          marginLeft: 10,
        }}
      >
        {info.row.original.isDirectory && <Folder2 size={20} />}
        {!info.row.original.isDirectory && <MusicNoteBeamed size={20} />}
      </div>
    ),
  }),
  columnHelper.accessor("name", {
    header: "",
    cell: (info) => (
      <>
        {info.row.original.isDirectory && (
          <Directory to={`/files?q=${info.row.original.path}`}>
            {info.getValue()}
          </Directory>
        )}
        {!info.row.original.isDirectory && (
          <AudioFile>{info.getValue()}</AudioFile>
        )}
      </>
    ),
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
  canGoBack: boolean;
  onGoBack: () => void;
  refetching?: boolean;
};

const Files: FC<FilesProps> = (props) => {
  return (
    <Container>
      <Sidebar active="files" />
      <MainView>
        <ControlBar />
        <ContentWrapper>
          {props.canGoBack && (
            <BackButton onClick={() => props.onGoBack()}>
              <div style={{ marginTop: 2 }}>
                <ArrowBack color={"#000"} />
              </div>
            </BackButton>
          )}
          <Title>Files</Title>
          {!props.refetching && (
            <Table columns={columns as any} tracks={props.files as any} />
          )}
          {props.refetching && (
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                height: "calc(100vh - 200px)",
              }}
            >
              <Spinner $size={"30px"} $borderWidth={"4px"} />
            </div>
          )}
        </ContentWrapper>
      </MainView>
    </Container>
  );
};

export default Files;
