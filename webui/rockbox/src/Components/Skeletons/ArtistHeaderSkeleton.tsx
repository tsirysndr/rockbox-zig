import { FC } from "react";
import ContentLoader from "react-content-loader";
import useSkeletonColors from "./useSkeletonColors";

const ArtistHeaderSkeleton: FC = () => {
  const { backgroundColor, foregroundColor } = useSkeletonColors();
  return (
    <ContentLoader
      speed={2}
      width="100%"
      height={200}
      viewBox="0 0 700 200"
      preserveAspectRatio="xMinYMid meet"
      backgroundColor={backgroundColor}
      foregroundColor={foregroundColor}
    >
      <circle cx="100" cy="100" r="100" />
      <rect x="220" y="75" rx="3" ry="3" width="260" height="28" />
      <rect x="220" y="115" rx="3" ry="3" width="180" height="16" />
    </ContentLoader>
  );
};

export default ArtistHeaderSkeleton;
