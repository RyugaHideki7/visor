"use client";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faChevronLeft, faMinus, faXmark } from "@fortawesome/free-solid-svg-icons";
import { faSquare as faSquareRegular } from "@fortawesome/free-regular-svg-icons";
import Image from "next/image";
import { motion, useReducedMotion } from "framer-motion";
import type { CSSProperties } from "react";
import { useState } from "react";
import { useSidebar } from "@/shared/contexts/sidebar";
import {
  closeWindow,
  minimizeWindow,
  toggleMaximizeWindow,
} from "@/shared/tauri/window";

const barColor = "var(--dark-bg-sidebar)";

export function Titlebar() {
  const [hovered, setHovered] = useState<"min" | "max" | "close" | null>(null);
  const { isExpanded, toggle } = useSidebar();
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
        background: barColor,
        height: 32,
        display: "flex",
        alignItems: "center",
        userSelect: "none",
        WebkitUserSelect: "none",
        borderBottom: "1px solid var(--dark-divider)",
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
          color: "var(--dark-text-secondary)",
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
            color: "var(--dark-text-secondary)",
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

        <Image
          src="/visor-logo-white.svg"
          alt="Visor"
          width={16}
          height={16}
          style={{ display: "block", borderRadius: 4 }}
        />
        <span style={{ fontWeight: 500 }}>Visor</span>
      </div>

      <div style={{ display: "flex" }}>
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
  color: "var(--dark-text-secondary)",
  cursor: "pointer",
};

const closeButtonStyle: CSSProperties = {
  color: "var(--dark-text-secondary)",
};
