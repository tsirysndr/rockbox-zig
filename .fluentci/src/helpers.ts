import { dag, Directory, DirectoryID } from "../deps.ts";

const exclude = [
  "build",
  ".zig-cache",
  "target",
  "zig-out",
  ".git",
  "tools/bmp2rb",
  "tools/codepages",
  "tools/convbdf",
  "tools/mkboot",
  "tools/rdfbinary",
  "tools/uclpack",
  "tools/scramble",
];

export const getDirectory = async (
  src: string | Directory | undefined = "."
) => {
  if (src instanceof Directory) {
    return src;
  }
  if (typeof src === "string") {
    try {
      const directory = dag.loadDirectoryFromID(src as DirectoryID);
      await directory.id();
      return directory;
    } catch (_) {
      return dag.host
        ? dag.host().directory(src, { exclude })
        : dag.currentModule().source().directory(src);
    }
  }
  return dag.host
    ? dag.host().directory(src, { exclude })
    : dag.currentModule().source().directory(src);
};
