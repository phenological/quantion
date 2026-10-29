import iconAddress from "../../../assets/icon.svg";
import { GitHubLinks } from "./GitHubLinks";
import { VersionMenu } from "./VersionMenu";

export function Header() {
  return (
    <header className="top">
      <img className="icon" src={iconAddress} alt="" />
      <div className="title">
        <h1 className="brand">
          Quant<span className="dot">·</span>ion
        </h1>
        <p className="tagline">The toolkit for Ionic files (.ion) · API documentation</p>
      </div>
      <VersionMenu />
      <GitHubLinks />
    </header>
  );
}
