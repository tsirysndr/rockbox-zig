import { FC, useRef } from "react";
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
  onSeek: (time: number) => void;
};

const CurrentTrack: FC<CurrentTrackProps> = ({
  nowPlaying,
  liked,
  onLike,
  onUnlike,
  onSeek,
}) => {
  const progressbarRef = useRef<HTMLDivElement>(null);
  const { formatTime } = useTimeFormat();
  const album = `${nowPlaying?.artist} - ${nowPlaying?.album}`;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const handleClick = (e: any) => {
    if (progressbarRef.current) {
      const rect = progressbarRef.current.getBoundingClientRect();
      const x = e.clientX - rect.left < 0 ? 0 : e.clientX - rect.left;
      const width = rect.width;
      const percentage = (x / width) * 100;
      const time = (percentage / 100) * nowPlaying!.duration;
      onSeek(Math.floor(time));
    }
  };

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
                {_.get(nowPlaying, "artist.length", 0) > 40
                  ? `${nowPlaying.artist?.substring(0, 40)}...`
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
            <ProgressbarContainer
              ref={progressbarRef}
              onClick={handleClick}
              active={nowPlaying!.duration > 0}
            >
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
