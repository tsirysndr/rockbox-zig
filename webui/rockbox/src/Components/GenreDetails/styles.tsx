import styled from "@emotion/styled";
import { Link as RouterLink } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const MainView = styled.div`
  display: flex;
  flex: 1;
  flex-direction: column;
  width: calc(100% - 240px);
`;

export const ContentWrapper = styled.div`
  padding-left: 30px;
  padding-right: 30px;
  overflow-y: auto;
  height: calc(100vh - 60px);
`;

export const BackButton = styled.button`
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 30px;
  width: 30px;
  border-radius: 15px;
  background-color: ${(props) => props.theme.colors.backButton};
  margin-top: 26px;
  margin-bottom: 18px;
  position: absolute;
  z-index: 1;
`;

export const GenreHero = styled.div<{ bg: string }>`
  position: relative;
  height: 220px;
  border-radius: 12px;
  overflow: hidden;
  margin-top: 30px;
  margin-bottom: 24px;
  background: linear-gradient(135deg, ${(p) => p.bg}, rgba(0, 0, 0, 0.4));
  display: flex;
  align-items: flex-end;
  padding: 22px 28px;
  color: white;
`;

export const GenreLabel = styled.div`
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 2px;
  opacity: 0.85;
`;

export const GenreName = styled.div`
  font-family: RockfordSansMedium;
  font-size: 38px;
  font-weight: 700;
  margin-top: 6px;
`;

export const GenreStats = styled.div`
  font-size: 12px;
  margin-top: 8px;
  opacity: 0.85;
`;

export const GenreDecoration = styled.div`
  position: absolute;
  right: -10px;
  bottom: -22px;
  font-family: RockfordSansMedium;
  font-size: 110px;
  font-weight: 700;
  opacity: 0.18;
  transform: rotate(-12deg);
  pointer-events: none;
`;

export const SectionTitle = styled.div`
  margin-top: 30px;
  margin-bottom: 14px;
  font-size: 20px;
  font-weight: 600;
`;

export const ButtonGroup = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 14px;
`;

export const Label = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

export const Link = styled(RouterLink)`
  color: ${(props) => props.theme.colors.text};
  text-decoration: none;
  &:hover {
    text-decoration: underline;
  }
`;

export const ArtistThumb = styled(RouterLink)`
  display: flex;
  flex-direction: column;
  align-items: center;
  text-decoration: none;
  color: inherit;
  cursor: pointer;
`;

export const ArtistImage = styled.img`
  width: 100px;
  height: 100px;
  border-radius: 50%;
  object-fit: cover;
`;

export const ArtistImagePlaceholder = styled.div`
  width: 100px;
  height: 100px;
  border-radius: 50%;
  background-color: ${(props) => props.theme.colors.cover};
  display: flex;
  align-items: center;
  justify-content: center;
`;

export const ArtistName = styled.div`
  margin-top: 8px;
  font-size: 13px;
  text-align: center;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;
