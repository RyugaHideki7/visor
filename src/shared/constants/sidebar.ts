export const SIDEBAR_WIDTH_EXPANDED = 270;
export const SIDEBAR_WIDTH_COLLAPSED = 72;

import {
  faChartSimple,
  faDatabase,
  faFolderOpen,
  faGaugeHigh,
  faGear,
  faIndustry,
} from "@fortawesome/free-solid-svg-icons";

export const SIDEBAR_ITEMS = [
  { key: "dashboard", label: "Tableau de bord", icon: faGaugeHigh, href: "/" },
  { key: "lines", label: "Lignes de production", icon: faIndustry, href: "/lines" },
  { key: "mapping", label: "Mapping", icon: faFolderOpen, href: "/mapping" },
  { key: "sql-queries", label: "Requêtes SQL", icon: faDatabase, href: "/sql-queries" },
  { key: "logs", label: "Journaux", icon: faChartSimple, href: "/journaux" },
  { key: "settings", label: "Paramètres", icon: faGear, href: "/settings" },
];
