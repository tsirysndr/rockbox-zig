import styled from "@emotion/styled";
import { LazyLoadImage } from "react-lazy-load-image-component";
import { Link as RouterLink } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const Scrollable = styled.div`
  height: calc(100vh - 60px);
  overflow-y: auto;
`;

export const Title = styled.div`
  font-size: 28px;
  font-family: RockfordSansMedium;
  margin-left: 30px;
  margin-top: 24px;
  margin-bottom: 24px;
`;

export const SectionTitle = styled.div`
  font-size: 20px;
  font-weight: 600;
  margin-left: 30px;
  margin-top: 30px;
  margin-bottom: 14px;
`;

export const Row = styled.div`
  display: flex;
  flex-direction: row;
  gap: 16px;
  padding-left: 30px;
  padding-right: 30px;
  overflow-x: auto;
  padding-bottom: 8px;
`;

export const Card = styled(RouterLink)`
  flex: 0 0 auto;
  width: 160px;
  text-decoration: none;
  color: inherit;
  cursor: pointer;
  &:hover .card-overlay {
    opacity: 1;
  }
`;

export const ArtistCard = styled(RouterLink)`
  flex: 0 0 auto;
  width: 130px;
  text-decoration: none;
  color: inherit;
  text-align: center;
  cursor: pointer;
`;

export const Cover = styled(LazyLoadImage)`
  width: 160px;
  height: 160px;
  border-radius: 4px;
  object-fit: cover;
`;

export const CoverPlaceholder = styled.div`
  width: 160px;
  height: 160px;
  border-radius: 4px;
  background-color: ${(p) => p.theme.colors.cover};
  display: flex;
  align-items: center;
  justify-content: center;
`;

export const ArtistImage = styled(LazyLoadImage)`
  width: 130px;
  height: 130px;
  border-radius: 50%;
  object-fit: cover;
`;

export const ArtistImagePlaceholder = styled.div`
  width: 130px;
  height: 130px;
  border-radius: 50%;
  background-color: ${(p) => p.theme.colors.cover};
  display: flex;
  align-items: center;
  justify-content: center;
`;

export const CardTitle = styled.div`
  margin-top: 8px;
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 160px;
`;

export const CardSubtitle = styled.div`
  margin-top: 2px;
  font-size: 12px;
  color: ${(p) => p.theme.colors.secondaryText};
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 160px;
`;

export const QuickPicksGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
  padding-left: 30px;
  padding-right: 30px;
  margin-bottom: 20px;
`;

export const QuickPickCard = styled(RouterLink)`
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background-color: ${(p) => p.theme.colors.cover};
  border-radius: 4px;
  text-decoration: none;
  color: inherit;
  cursor: pointer;
  &:hover {
    filter: brightness(1.15);
  }
`;

export const QuickPickThumb = styled.div`
  width: 48px;
  height: 48px;
  border-radius: 3px;
  background-color: ${(p) => p.theme.colors.backButton};
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
`;

export const QuickPickName = styled.div`
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

export const Empty = styled.div`
  padding: 30px;
  color: ${(p) => p.theme.colors.secondaryText};
  font-size: 14px;
`;
