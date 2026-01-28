"use client";

import { createContext, useContext, useMemo, useState, type PropsWithChildren } from "react";
import { SIDEBAR_WIDTH_COLLAPSED, SIDEBAR_WIDTH_EXPANDED } from "@/shared/constants/sidebar";

export type SidebarContextValue = {
  isExpanded: boolean;
  toggle: () => void;
  expand: () => void;
  collapse: () => void;
  width: number;
};

const SidebarContext = createContext<SidebarContextValue | undefined>(undefined);

export function SidebarProvider({ children }: PropsWithChildren) {
  const [isExpanded, setIsExpanded] = useState(true);

  const value = useMemo<SidebarContextValue>(
    () => ({
      isExpanded,
      width: isExpanded ? SIDEBAR_WIDTH_EXPANDED : SIDEBAR_WIDTH_COLLAPSED,
      toggle: () => setIsExpanded((prev) => !prev),
      expand: () => setIsExpanded(true),
      collapse: () => setIsExpanded(false),
    }),
    [isExpanded],
  );

  return <SidebarContext.Provider value={value}>{children}</SidebarContext.Provider>;
}

export function useSidebar() {
  const ctx = useContext(SidebarContext);
  if (!ctx) {
    throw new Error("useSidebar must be used within SidebarProvider");
  }
  return ctx;
}
