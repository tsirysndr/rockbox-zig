import styled from "@emotion/styled";
import { Link as RouterLink } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const AlbumCover = styled.img`
  height: 240px;
  width: 240px;
  border-radius: 3px;
`;

export const ContentWrapper = styled.div`
  padding-left: 30px;
  padding-right: 30px;
  overflow-y: auto;
  height: calc(100vh - 60px);
`;

export const AlbumTitle = styled.div`
  font-size: 32px;
  font-family: RockfordSansBold;
  color: ${(props) => props.theme.colors.text};
`;

export const Artist = styled(RouterLink)`
  color: ${(props) => props.theme.colors.text};
  text-decoration: none;
  font-family: RockfordSansMedium;
  font-size: 14px;
  margin-top: 8px;
  &:hover {
    text-decoration: underline;
  }
`;

export const Tracks = styled.div`
  margin-top: 25px;
  font-weight: 400;
  font-size: 14px;
  color: ${(props) => props.theme.colors.secondaryText};
`;

export const Year = styled.div`
  margin-top: 15px;
  font-weight: 400;
  font-size: 14px;
  margin-bottom: 10px;
  color: ${(props) => props.theme.colors.secondaryText};
`;

export const Header = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  margin-bottom: 20px;
  margin-top: 90px;
`;

export const AlbumInfos = styled.div`
  display: flex;
  flex-direction: column;
  margin-left: 26px;
  height: 240px;
  justify-content: center;
`;

export const ButtonGroup = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
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

export const IconButton = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  &:hover {
    opacity: 0.6;
  }
`;

export const Hover = styled.button`
  color: transparent;
  background-color: transparent;
  border: none;
  opacity: 1 !important;
  cursor: pointer;
  &:hover,
  &:focus {
    color: ${(props) => props.theme.colors.text};
    opacity: 1 !important;
  }
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
