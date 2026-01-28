export const SIDEBAR_WIDTH_EXPANDED = 240;
export const SIDEBAR_WIDTH_COLLAPSED = 72;

import {
  faChartSimple,
  faFolderOpen,
  faGaugeHigh,
  faGear,
} from "@fortawesome/free-solid-svg-icons";

export const SIDEBAR_ITEMS = [
  { key: "dashboard", label: "Dashboard", icon: faGaugeHigh, href: "/" },
  { key: "projects", label: "Projects", icon: faFolderOpen, href: "/projects" },
  { key: "activity", label: "Activity", icon: faChartSimple, href: "/activity" },
  { key: "settings", label: "Settings", icon: faGear, href: "/settings" },
];
