export const SIDEBAR_WIDTH_EXPANDED = 270;
export const SIDEBAR_WIDTH_COLLAPSED = 72;

import {
  faChartSimple,
  faDatabase,
  faFolderOpen,
  faGaugeHigh,
  faGear,
  faIndustry,
  faRightLeft,
  faServer,
  faBoxOpen,
  faListCheck,
  faNetworkWired,
} from "@fortawesome/free-solid-svg-icons";

export interface SidebarItem {
  key: string;
  label: string;
  icon: any;
  href?: string;
  children?: SidebarItem[];
}

export const SIDEBAR_MENU: SidebarItem[] = [
  { key: "dashboard", label: "Tableau de bord", icon: faGaugeHigh, href: "/" },
  {
    key: "production",
    label: "Remontée de production",
    icon: faIndustry,
    children: [
      { key: "lines", label: "Lignes de production", icon: faIndustry, href: "/lines" },
      { key: "mapping", label: "Mapping", icon: faFolderOpen, href: "/mapping" },
      { key: "sql-queries", label: "Requêtes SQL", icon: faDatabase, href: "/sql-queries" },
      { key: "logs", label: "Journaux", icon: faChartSimple, href: "/journaux" },
    ],
  },
  {
    key: "data-exchange",
    label: "Data exchange",
    icon: faRightLeft,
    children: [
      {
        key: "data-exchange-logitron",
        label: "Logitron",
        icon: faServer,
        children: [
          {
            key: "data-exchange-logitron-produit",
            label: "Produit",
            icon: faBoxOpen,
            href: "/data-exchange/logitron/produit",
          },
          {
            key: "data-exchange-logitron-of",
            label: "OF",
            icon: faListCheck,
            href: "/data-exchange/logitron/of",
          },
        ],
      },
      {
        key: "data-exchange-ateis",
        label: "Ateis",
        icon: faNetworkWired,
        href: "/data-exchange/ateis",
      },
    ],
  },
  { key: "settings", label: "Paramètres", icon: faGear, href: "/settings" },
];
