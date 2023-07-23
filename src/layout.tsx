import { Outlet, Link, useLocation } from "react-router-dom";
import { VscHome, VscRepoForked, VscSettingsGear } from "react-icons/vsc";
import clsx from "clsx";
import { NavigationProvider } from "./context/NavigationProvider";

export default function Layout() {
  const location = useLocation();

  const selectedClass = "text-primary";
  const defaultClass = "w-10 h-7";
  const linkClass = "hover:text-primary w-10 h-10";

  return (
    <NavigationProvider>
      <div className="flex flex-row min-w-screen min-h-screen overflow-hidden">
        <div className="w-14 bg-base-200 flex flex-col gap-3 px-2 pt-3">
          <Link className={linkClass} to="/">
            <VscHome
              className={clsx(defaultClass, {
                [selectedClass]: location.pathname === "/",
              })}
            />
          </Link>
          <Link className={linkClass} to="/flows">
            <VscRepoForked
              className={clsx(defaultClass, {
                [selectedClass]: location.pathname.includes("/drag"),
              })}
            />
          </Link>
          <div className="flex-grow" />
          <Link className={linkClass} to="/settings">
            <VscSettingsGear
              className={clsx(defaultClass, {
                [selectedClass]: location.pathname === "/settings",
              })}
            />
          </Link>
        </div>
        <div className="w-screen h-screen">
          <Outlet />
        </div>
      </div>
    </NavigationProvider>
  );
}
