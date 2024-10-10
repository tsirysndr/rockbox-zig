import { FC } from "react";
import Files from "./Files";
import { files } from "./mocks";

const FilesWithData: FC = () => {
  return <Files files={files} />;
};

export default FilesWithData;
