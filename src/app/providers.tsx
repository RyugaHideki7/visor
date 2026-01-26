"use client";

import { HeroUIProvider } from "@heroui/react";
import { library, config as faConfig } from "@fortawesome/fontawesome-svg-core";
import {
  faCheck,
  faCircleInfo,
  faTriangleExclamation,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import { faCircle as faCircleRegular } from "@fortawesome/free-regular-svg-icons";
import { faGithub, faTwitter } from "@fortawesome/free-brands-svg-icons";
import "@fortawesome/fontawesome-svg-core/styles.css";
import type { PropsWithChildren } from "react";
import { Titlebar } from "@/components/Titlebar";

faConfig.autoAddCss = false;
library.add(
  faCheck,
  faCircleInfo,
  faTriangleExclamation,
  faXmark,
  faCircleRegular,
  faGithub,
  faTwitter,
);

export function Providers({ children }: PropsWithChildren) {
  return (
    <HeroUIProvider>
      <div style={{ minHeight: "100vh", display: "flex", flexDirection: "column" }}>
        <Titlebar />
        <div style={{ flex: 1, minHeight: 0 }}>{children}</div>
      </div>
    </HeroUIProvider>
  );
}
