import { FC } from "react";
import ContentLoader from "react-content-loader";
import useSkeletonColors from "./useSkeletonColors";

const AlbumCardSkeleton: FC = () => {
  const { backgroundColor, foregroundColor } = useSkeletonColors();
  return (
    <ContentLoader
      speed={2}
      width="100%"
      viewBox="0 0 200 260"
      backgroundColor={backgroundColor}
      foregroundColor={foregroundColor}
    >
      <rect x="0" y="0" rx="3" ry="3" width="200" height="200" />
      <rect x="0" y="210" rx="3" ry="3" width="150" height="14" />
      <rect x="0" y="230" rx="3" ry="3" width="110" height="12" />
      <rect x="0" y="248" rx="3" ry="3" width="70" height="10" />
    </ContentLoader>
  );
};

export default AlbumCardSkeleton;
