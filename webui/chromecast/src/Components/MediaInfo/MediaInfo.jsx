const MediaInfo = ({ artist, title, images }) => {
  return (
    <div className="px-[5vw] flex">
      <img
        src={images.length ? images[0].url : ""}
        alt={title}
        className="h-[30vh] w-[30vh]"
      />
      <div className="pl-10 flex flex-col justify-end h-[30vh]">
        <div>
          <div className="font-rockford-bold text-[2.5em]">{title}</div>
          <div className="text-[18px]">{artist}</div>
        </div>
      </div>
    </div>
  );
};

MediaInfo.defaultProps = {
  artist: "",
  title: "",
  images: [],
};

export default MediaInfo;
