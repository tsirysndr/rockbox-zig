import styled from "@emotion/styled";
import { LazyLoadImage } from "react-lazy-load-image-component";

export const Container = styled.div`
  height: calc(100vh - 113px);
  width: 370px;
`;

export const Header = styled.div`
  display: flex;
  flex-direction: row;
`;

export const Title = styled.div`
  font-size: 14px;
  margin-left: 16px;
  margin-right: 16px;
  padding-top: 20px;
  margin-bottom: 20px;
  flex: 1;
`;

export const Switch = styled(Title)`
  color: #fe099c;
  flex: 1;
  text-align: end;
  cursor: pointer;
  -webkit-user-select: none;
  -ms-user-select: none;
  user-select: none;
`;

export const List = styled.div`
  height: calc(100% - 59.5px);
  overflow-y: auto;
`;

export const ListItem = styled.div`
  display: flex;
  flex-direction: row;
  height: 64px;
  align-items: center;
  padding-left: 16px;
  padding-right: 16px;
  cursor: pointer;
  &:hover {
    background-color: ${({ theme }) => theme.colors.hover};
  }
`;

export const TrackTitle = styled.div`
  font-size: 14px;
  font-family: RockfordSansMedium;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
`;

export const Artist = styled.div`
  font-size: 14px;
  color: ${({ theme }) => theme.colors.secondaryText};
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
`;

export const TrackDetails = styled.div`
  display: flex;
  min-width: 222px;
  flex-direction: column;
  flex: 1;
`;

export const AlbumCover = styled(LazyLoadImage)<{ current?: boolean }>`
  height: 48px;
  width: 48px;
  border-radius: 4px;
  margin-right: 18px;
  cursor: pointer;
  ${({ current }) => `opacity: ${current ? 0.4 : 1};`}
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
  margin-right: 18px;
  ${({ current }) => `opacity: ${current ? 0 : 1};`}
`;

export const Remove = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
`;

export const Placeholder = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  padding-left: 20px;
  padding-right: 20px;
  font-size: 14px;
`;
