import { useTheme } from "next-themes";

export default function DownArrow() {
  const { theme, setTheme } = useTheme();
  if (theme == "light") {
    return <i className="arrow down" style={{ borderColor: "black", borderWidth: "0 4px 4px 0" }}></i>;
  } else {
    return <i className="arrow down" style={{ borderColor: "white", borderWidth: "0 4px 4px 0" }}></i>;
  }
}
