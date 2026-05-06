import { useContext, useEffect, useState } from "react";
import { CastContext } from "./CastProvider";
import Header from "./Components/Header";
import MediaInfo from "./Components/MediaInfo";
import Progress from "./Components/Progress";
import Splash from "./Components/Splash";
import { BehaviorSubject } from "rxjs";

const cast = window.cast;

const CustomReceiver = () => {
  const [track, setTrack] = useState({
    artist: "",
    title: "",
    images: [],
  });
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const request$ = new BehaviorSubject(null);
  const { playerManager } = useContext(CastContext);

  useEffect(() => {
    if (playerManager) {
      playerManager.addEventListener(
        cast.framework.events.EventType.MEDIA_STATUS,
        (request) => {
          request$.next({
            ...request,
            eventType: cast.framework.events.EventType.MEDIA_STATUS,
          });
        }
      );

      playerManager.addEventListener(
        cast.framework.events.EventType.TIME_UPDATE,
        (request) => {
          request$.next({
            ...request,
            eventType: cast.framework.events.EventType.TIME_UPDATE,
          });
        }
      );
    }
  }, [playerManager]);

  useEffect(() => {
    const subscription = request$.subscribe((request) => {
      if (
        request &&
        request.mediaStatus &&
        request.mediaStatus.playerState === "PLAYING" &&
        request.eventType === cast.framework.events.EventType.MEDIA_STATUS
      ) {
        setIsLoading(false);
        setTrack(request.mediaStatus.media.metadata);
        setDuration(request.mediaStatus.media.duration);
      }
      if (
        request &&
        request.eventType === cast.framework.events.EventType.TIME_UPDATE
      ) {
        setCurrentTime(request.currentMediaTime);
      }
    });
    return () => subscription.unsubscribe();
  }, []);

  const coverUrl = track.images.length ? track.images[0].url : null;

  return (
    <>
      {isLoading && <Splash />}
      {!isLoading && (
        <div
          className="bg-center bg-no-repeat bg-cover h-screen w-full"
          style={coverUrl ? { backgroundImage: `url(${coverUrl})` } : { backgroundColor: '#080808' }}
        >
          <div className="bg-black/70 h-screen">
            <Header />
            <div className="w-full h-[calc(100vh-70px-54vh)]" />
            <MediaInfo {...track} />
            <Progress
              currentTime={currentTime * 1000}
              duration={duration * 1000}
            />
          </div>
        </div>
      )}
    </>
  );
};

export default CustomReceiver;
