import styled from "@emotion/styled";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  height: 45px;
`;

export const Separator = styled.div`
  width: 10px;
`;

export const Hover = styled.button`
  color: transparent;
  background-color: transparent;
  border: none;
  &:hover,
  &:focus {
    color: #000;
  }
`;

export const Icon = styled.div`
  cursor: pointer;
  display: flex;
  height: 45px;
  width: 24px;
  justify-content: center;
  align-items: center;
`;

export const AlbumCover = styled.img`
  height: 43px;
  width: 43px;
`;

export const AlbumCoverAlt = styled.div`
  height: 43px;
  width: 43px;
  background-color: ${(props) => props.theme.colors.cover};
  display: flex;
  justify-content: center;
  align-items: center;
`;

export const Track = styled.div`
  height: 54px;
  display: flex;
  flex-direction: row;
  align-items: center;
  padding-left: 5px;
  padding-right: 5px;
  border-bottom: 1px solid ${(props) => props.theme.colors.separator};
`;

export const Artist = styled.div`
  color: rgb(170, 170, 180);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
  overflow: hidden;
  max-width: 125px;
`;

export const Title = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
  overflow: hidden;
  max-width: 125px;
`;

export const TrackInfos = styled.div`
  margin-left: 10px;
  overflow: hidden;
`;
