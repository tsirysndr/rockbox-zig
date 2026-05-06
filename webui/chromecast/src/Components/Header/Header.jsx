import Rockbox from "../../Assets/rockbox-icon.png";

const Header = () => {
  return (
    <div className="h-[70px] w-full px-[5vw] flex items-center mt-5">
      <div className="flex flex-row items-center h-[70px]">
        <img src={Rockbox} alt="Rockbox" className="h-10 w-10" />
        <div className="font-rockford-bold ml-2.5">Rockbox</div>
      </div>
    </div>
  );
};

export default Header;
