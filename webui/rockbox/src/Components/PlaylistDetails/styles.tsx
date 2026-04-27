import styled from "@emotion/styled";
import { Link as RouterLink } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const ContentWrapper = styled.div`
  padding-left: 30px;
  padding-right: 30px;
  overflow-y: auto;
  height: calc(100vh - 60px);
`;

export const Header = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  margin-bottom: 20px;
  margin-top: 90px;
`;

export const CoverArt = styled.div<{ image?: string }>`
  height: 240px;
  width: 240px;
  border-radius: 6px;
  background-color: ${(props) => props.theme.colors.cover};
  background-image: ${({ image }) => (image ? `url(${image})` : "none")};
  background-size: cover;
  background-position: center;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
`;

export const PlaylistInfos = styled.div`
  display: flex;
  flex-direction: column;
  margin-left: 26px;
  height: 240px;
  justify-content: center;
`;

export const PlaylistTitle = styled.div`
  font-size: 32px;
  font-family: RockfordSansBold;
`;

export const PlaylistDescription = styled.div`
  font-size: 14px;
  color: #555;
  margin-top: 8px;
`;

export const TrackCount = styled.div`
  margin-top: 25px;
  font-weight: 400;
  font-size: 14px;
`;

export const ButtonGroup = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  margin-top: 20px;
`;

export const Separator = styled.div`
  width: 20px;
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
  margin-bottom: 46px;
  position: absolute;
  z-index: 1;
`;

export const Label = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

export const Link = styled(RouterLink)`
  color: ${(props) => props.theme.colors.text};
  text-decoration: none;
  font-family: RockfordSansRegular;
  &:hover {
    text-decoration: underline;
  }
`;
