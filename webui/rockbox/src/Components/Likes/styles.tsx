import styled from "@emotion/styled";
import { LazyLoadImage } from "react-lazy-load-image-component";
import { Link as RouterLink } from "react-router-dom";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const Title = styled.div`
  font-size: 24px;
  font-family: RockfordSansMedium;
  margin-bottom: 20px;
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
    color: #000;
    opacity: 1 !important;
  }
`;

export const ButtonGroup = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

export const ContentWrapper = styled.div`
  overflow-y: auto;
  height: calc(100vh - 60px);
  padding-left: 20px;
  padding-right: 20px;
  position: relative;
`;

export const AlbumCover = styled(LazyLoadImage)`
  height: 48px;
  width: 48px;
`;

export const AlbumCoverAlt = styled.div<{ current?: boolean }>`
  height: 48px;
  width: 48px;
  border-radius: 4px;
  cursor: pointer;
  background-color: ${(props) => props.theme.colors.cover};
  display: flex;
  justify-content: center;
  align-items: center;
  ${({ current }) => `opacity: ${current ? 0 : 1};`}
`;

export const FilterContainer = styled.div`
  margin-top: 30px;
  margin-bottom: 40px;
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
  background-color: #f7f7f8;
  margin-top: 26px;
  margin-bottom: 46px;
  position: absolute;
  z-index: 1;
`;

export const Link = styled(RouterLink)`
  color: #000;
  text-decoration: none;
  font-family: RockfordSansRegular;
  &:hover {
    text-decoration: underline;
  }
`;

export const Label = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

export const Separator = styled.div`
  width: 20px;
`;

export const HeaderWrapper = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
`;
