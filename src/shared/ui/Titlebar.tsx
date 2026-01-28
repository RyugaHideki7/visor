"use client";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faMinus, faXmark } from "@fortawesome/free-solid-svg-icons";
import { faSquare as faSquareRegular } from "@fortawesome/free-regular-svg-icons";
import type { CSSProperties } from "react";
import { useState } from "react";
import {
  closeWindow,
  minimizeWindow,
  toggleMaximizeWindow,
} from "@/shared/tauri/window";

const barColor = "var(--dark-bg-sidebar)";

export function Titlebar() {
  const [hovered, setHovered] = useState<"min" | "max" | "close" | null>(null);

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
        <img
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
