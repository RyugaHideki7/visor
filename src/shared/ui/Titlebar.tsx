"use client";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faChevronLeft, faMinus, faXmark, faSun, faMoon } from "@fortawesome/free-solid-svg-icons";
import { faSquare as faSquareRegular } from "@fortawesome/free-regular-svg-icons";
import { motion, useReducedMotion } from "framer-motion";
import type { CSSProperties } from "react";
import { useState } from "react";
import { useSidebar } from "@/shared/contexts/sidebar";
import { useTheme } from "@/shared/contexts/theme";
import { VisorLogo } from "@/shared/ui/VisorLogo";
import {
  closeWindow,
  minimizeWindow,
  toggleMaximizeWindow,
} from "@/shared/tauri/window";

export function Titlebar() {
  const [hovered, setHovered] = useState<"theme" | "min" | "max" | "close" | null>(null);
  const { isExpanded, toggle } = useSidebar();
  const { theme, toggleTheme } = useTheme();
  const prefersReducedMotion = useReducedMotion();

  const handleMinimize = async () => {
    await minimizeWindow();
  };

  const handleMaximize = async () => {
    await toggleMaximizeWindow();
  };

  const handleClose = async () => {
    await closeWindow();
  };

  return (
    <div
      data-tauri-drag-region
      style={{
        background: "var(--bg-secondary)",
        height: 32,
        display: "flex",
        alignItems: "center",
        userSelect: "none",
        WebkitUserSelect: "none",
        borderBottom: "1px solid var(--border-default)",
      }}
    >
      <div
        data-tauri-drag-region
        style={{
          flex: 1,
          display: "flex",
          alignItems: "center",
          gap: 10,
          paddingLeft: 12,
          color: "var(--text-secondary)",
          fontSize: 12,
        }}
      >
        <motion.button
          type="button"
          aria-label={isExpanded ? "Collapse sidebar" : "Expand sidebar"}
          data-tauri-drag-region="false"
          onClick={toggle}
          whileTap={{ scale: prefersReducedMotion ? 1 : 0.95 }}
          whileHover={{ scale: prefersReducedMotion ? 1 : 1.02 }}
          transition={{ type: "spring", stiffness: 400, damping: 30 }}
          style={{
            background: "transparent",
            border: "none",
            display: "grid",
            placeItems: "center",
            width: 28,
            height: 28,
            color: "var(--text-secondary)",
            cursor: "pointer",
          }}
        >
          <motion.span
            animate={{ rotate: isExpanded ? 0 : 180 }}
            transition={{ duration: prefersReducedMotion ? 0 : 0.25, ease: "easeOut" }}
            style={{ display: "grid", placeItems: "center" }}
          >
            <FontAwesomeIcon icon={faChevronLeft} className="h-3 w-3" />
          </motion.span>
        </motion.button>

        <VisorLogo
          variant={theme === "light" ? "color" : "white"}
          size={16}
          className="rounded"
        />
        <span style={{ fontWeight: 500 }}>Visor</span>
      </div>

      <div style={{ display: "flex", alignItems: "center" }}>
        {/* Theme Toggle */}
        <button
          aria-label={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
          onClick={toggleTheme}
          data-tauri-drag-region="false"
          style={{
            ...winButtonStyle,
            background: hovered === "theme" ? "var(--bg-hover)" : "transparent",
          }}
          onMouseEnter={() => setHovered("theme")}
          onMouseLeave={() => setHovered(null)}
        >
          <FontAwesomeIcon icon={theme === "dark" ? faSun : faMoon} className="h-3 w-3" />
        </button>
        
        <button
          aria-label="Minimize"
          onClick={handleMinimize}
          data-tauri-drag-region="false"
          style={{
            ...winButtonStyle,
            background: hovered === "min" ? "rgba(255,255,255,0.06)" : "transparent",
          }}
          onMouseEnter={() => setHovered("min")}
          onMouseLeave={() => setHovered(null)}
        >
          <FontAwesomeIcon icon={faMinus} className="h-2.5 w-2.5" />
        </button>
        <button
          aria-label="Maximize"
          onClick={handleMaximize}
          data-tauri-drag-region="false"
          style={{
            ...winButtonStyle,
            background: hovered === "max" ? "rgba(255,255,255,0.06)" : "transparent",
          }}
          onMouseEnter={() => setHovered("max")}
          onMouseLeave={() => setHovered(null)}
        >
          <FontAwesomeIcon icon={faSquareRegular} className="h-2.5 w-2.5" />
        </button>
        <button
          aria-label="Close"
          onClick={handleClose}
          data-tauri-drag-region="false"
          style={{
            ...winButtonStyle,
            ...closeButtonStyle,
            background: hovered === "close" ? "#e81123" : "transparent",
          }}
          onMouseEnter={() => setHovered("close")}
          onMouseLeave={() => setHovered(null)}
        >
          <FontAwesomeIcon icon={faXmark} className="h-2.5 w-2.5" />
        </button>
      </div>
    </div>
  );
}

const winButtonStyle: CSSProperties = {
  width: 46,
  height: 32,
  display: "grid",
  placeItems: "center",
  background: "transparent",
  border: "none",
  color: "var(--text-secondary)",
  cursor: "pointer",
};

const closeButtonStyle: CSSProperties = {
  color: "var(--text-secondary)",
};
