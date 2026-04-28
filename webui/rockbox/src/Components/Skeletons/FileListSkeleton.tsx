import { FC } from "react";
import ContentLoader from "react-content-loader";
import useSkeletonColors from "./useSkeletonColors";

const FileRowSkeleton: FC = () => {
  const { backgroundColor, foregroundColor } = useSkeletonColors();
  return (
    <ContentLoader
      speed={2}
      width="100%"
      height={48}
      viewBox="0 0 1000 48"
      preserveAspectRatio="none"
      backgroundColor={backgroundColor}
      foregroundColor={foregroundColor}
    >
      <rect x="20" y="14" rx="2" ry="2" width="20" height="20" />
      <rect x="54" y="17" rx="2" ry="2" width="300" height="14" />
    </ContentLoader>
  );
};

type Props = { rows?: number };

const FileListSkeleton: FC<Props> = ({ rows = 12 }) => (
  <div>
    {Array.from({ length: rows }).map((_, i) => (
      <FileRowSkeleton key={i} />
    ))}
  </div>
);

export default FileListSkeleton;
