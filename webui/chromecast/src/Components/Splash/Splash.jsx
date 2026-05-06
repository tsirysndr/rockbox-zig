import RockboxLogo from "../../Assets/rockbox-logo.svg";

const Splash = () => {
  return (
    <div className="bg-black h-screen w-full flex justify-center items-center">
      <img src={RockboxLogo} alt="Rockbox" />
      <div className="absolute bottom-0 flex justify-center items-center h-[20vh] w-full">
        <div className="h-10 w-10 rounded-full border-4 border-white border-t-transparent animate-spin" />
      </div>
    </div>
  );
};

export default Splash;
