import { FC } from "react";
import { ProgressBar } from "baseui/progress-bar";
import {
  Actions,
  Album,
  AlbumCover,
  ArtistAlbum,
  Container,
  Icon,
  NoCover,
  ProgressbarContainer,
  Separator,
  styles,
  Time,
  Title,
  TrackInfo,
} from "./styles";
import Track from "../../Icons/Track";
import { useTimeFormat } from "../../../Hooks/useFormat";
import { CurrentTrack as NowPlaying } from "../../../Types/track";
import _ from "lodash";
import HeartOutline from "../../Icons/HeartOutline";
import Heart from "../../Icons/Heart";

export type CurrentTrackProps = {
  nowPlaying?: NowPlaying;
  liked?: boolean;
  onLike: (trackId: string) => void;
  onUnlike: (trackId: string) => void;
};

const CurrentTrack: FC<CurrentTrackProps> = ({
  nowPlaying,
  liked,
  onLike,
  onUnlike,
}) => {
  const { formatTime } = useTimeFormat();
  const album = `${nowPlaying?.artist} - ${nowPlaying?.album}`;
  return (
    <Container>
      {!nowPlaying?.cover && (
        <NoCover>
          <Track color="#b1b2b5" height={28} width={28} />
        </NoCover>
      )}
      {nowPlaying?.cover && (
        <AlbumCover src={nowPlaying.cover} alt="Album cover" />
      )}
      <TrackInfo>
        {(!nowPlaying || nowPlaying?.duration === 0) && (
          <div style={{ color: "#b1b2b5", textAlign: "center" }}>
            No track playing
          </div>
        )}
        {nowPlaying && nowPlaying?.duration > 0 && (
          <>
            <div style={{ display: "flex", flexDirection: "row" }}>
              <Actions />
              <Title>
                {_.get(nowPlaying, "title.length", 0) > 75
                  ? `${nowPlaying.title?.substring(0, 75)}...`
                  : nowPlaying.title}
              </Title>
              <Actions>
                {!liked && (
                  <Icon onClick={() => onLike(nowPlaying!.id!)}>
                    <HeartOutline color="#000" />
                  </Icon>
                )}
                {liked && (
                  <Icon onClick={() => onUnlike(nowPlaying!.id!)}>
                    <Heart color="#fe09a3" />
                  </Icon>
                )}
              </Actions>
            </div>
            <div
              style={{
                display: "flex",
                flexDirection: "row",
                alignItems: "center",
                justifyContent: "space-between",
              }}
            >
              <Time>{formatTime(nowPlaying.progress)}</Time>
              <ArtistAlbum>
                {_.get(nowPlaying, "artist.length", 0) > 65
                  ? `${nowPlaying.artist?.substring(0, 54)}...`
                  : nowPlaying.artist}
                <Separator>-</Separator>
                <Album to={`/albums/${nowPlaying.albumId}`}>
                  {album.length > 75
                    ? `${nowPlaying.album?.substring(0, 30)}...`
                    : nowPlaying.album}
                </Album>
              </ArtistAlbum>
              <Time>{formatTime(nowPlaying.duration)}</Time>
            </div>
            <ProgressbarContainer>
              <ProgressBar
                value={
                  nowPlaying!.duration > 0
                    ? (nowPlaying!.progress / nowPlaying!.duration) * 100
                    : 0
                }
                overrides={styles.Progressbar}
              />
            </ProgressbarContainer>
          </>
        )}
      </TrackInfo>
    </Container>
  );
};

export default CurrentTrack;
