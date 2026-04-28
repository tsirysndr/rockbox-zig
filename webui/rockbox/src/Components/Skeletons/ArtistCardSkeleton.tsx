import { FC } from "react";
import ContentLoader from "react-content-loader";
import useSkeletonColors from "./useSkeletonColors";

const ArtistCardSkeleton: FC = () => {
  const { backgroundColor, foregroundColor } = useSkeletonColors();
  return (
    <ContentLoader
      speed={2}
      width={194}
      height={230}
      viewBox="0 0 194 230"
      backgroundColor={backgroundColor}
      foregroundColor={foregroundColor}
    >
      <circle cx="97" cy="97" r="97" />
      <rect x="22" y="208" rx="3" ry="3" width="150" height="14" />
    </ContentLoader>
  );
};

export default ArtistCardSkeleton;
