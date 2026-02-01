"use client";

import React from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faChevronDown, faIndustry } from "@fortawesome/free-solid-svg-icons";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useMemo, useState } from "react";
import { SIDEBAR_ITEMS } from "@/shared/constants/sidebar";
import { useSidebar } from "@/shared/contexts/sidebar";
import { useTheme } from "@/shared/contexts/theme";
import { VisorLogo } from "@/shared/ui/VisorLogo";

export function Sidebar() {
  const { isExpanded, width } = useSidebar();
  const { theme } = useTheme();
  const prefersReducedMotion = useReducedMotion();
  const pathname = usePathname();
  const [isProductionOpen, setIsProductionOpen] = useState(true);

  // Group production-related pages: lines, mapping, sql-queries, journaux
  const productionKeys = useMemo(() => new Set(["lines", "mapping", "sql-queries", "logs"]), []);

  const containerStyle = useMemo(
    () => ({
      background: "var(--bg-secondary)",
      borderRight: "1px solid var(--border-default)",
      color: "var(--text-secondary)",
      display: "flex",
      flexDirection: "column" as const,
      padding: isExpanded ? "12px 10px" : "12px 8px",
      gap: 10,
      overflow: "hidden",
      willChange: "width",
    }),
    [isExpanded],
  );

  return (
    <motion.aside
      initial={false}
      animate={{ width }}
      transition={
        prefersReducedMotion
          ? { duration: 0 }
          : { type: "spring", stiffness: 320, damping: 34 }
      }
      style={containerStyle}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: isExpanded ? "flex-start" : "center",
          gap: 10,
          padding: isExpanded ? "6px 10px" : "6px 6px",
        }}
      >
        <VisorLogo variant={theme === "light" ? "color" : "white"} size={32} />
        <AnimatePresence initial={false}>
          {isExpanded && (
            <motion.span
              key="brand"
              initial={{ opacity: 0, x: -8 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -8 }}
              transition={{ duration: prefersReducedMotion ? 0 : 0.18, ease: "easeOut" }}
              style={{ fontWeight: 700, fontSize: 15, color: "var(--text-primary)" }}
            >
              Visor
            </motion.span>
          )}
        </AnimatePresence>
      </div>

      <nav
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 4,
          alignItems: isExpanded ? "stretch" : "center",
        }}
      >
        {(() => {
          let productionRendered = false;
          const out: React.ReactElement[] = [];

          for (const item of SIDEBAR_ITEMS) {
            if (productionKeys.has(item.key)) {
              if (productionRendered) {
                continue;
              }
              productionRendered = true;

              const isGroupActive = SIDEBAR_ITEMS.some((i) => productionKeys.has(i.key) && pathname === i.href);

              out.push(
                <div key="production-group" style={{ width: "100%" }}>
                  <button
                    type="button"
                    title={!isExpanded ? "Remontée de production" : undefined}
                    onClick={() => setIsProductionOpen((v) => !v)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      gap: 8,
                      padding: isExpanded ? "10px 12px" : "10px",
                      borderRadius: 8,
                      border: "none",
                      width: "100%",
                      minHeight: 44,
                      background: isGroupActive ? "var(--bg-active)" : "transparent",
                      color: isGroupActive ? "var(--text-primary)" : "var(--text-secondary)",
                      cursor: "pointer",
                      transition: "background 0.15s ease, color 0.15s ease",
                    }}
                    onMouseEnter={(e) => {
                      if (!isGroupActive) {
                        e.currentTarget.style.background = "var(--bg-hover)";
                        e.currentTarget.style.color = "var(--text-primary)";
                      }
                    }}
                    onMouseLeave={(e) => {
                      if (!isGroupActive) {
                        e.currentTarget.style.background = "transparent";
                        e.currentTarget.style.color = "var(--text-secondary)";
                      }
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: 10, minWidth: 0 }}>
                      <FontAwesomeIcon icon={faIndustry} className="h-4 w-4" style={{ flexShrink: 0 }} />
                      <AnimatePresence initial={false}>
                        {isExpanded && (
                          <motion.span
                            key="label"
                            initial={{ opacity: 0, x: -6 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -6 }}
                            transition={{ duration: prefersReducedMotion ? 0 : 0.15, ease: "easeOut" }}
                            style={{ fontSize: 13, fontWeight: 600, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}
                          >
                            Remontée de production
                          </motion.span>
                        )}
                      </AnimatePresence>
                    </div>
                    {isExpanded && (
                      <motion.span
                        animate={{ rotate: isProductionOpen ? 0 : -90 }}
                        transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.18, ease: "easeOut" }}
                        style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: 16 }}
                      >
                        <FontAwesomeIcon icon={faChevronDown} className="h-3 w-3" />
                      </motion.span>
                    )}
                  </button>

                  <AnimatePresence initial={false}>
                    {isProductionOpen && (
                      <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: "auto", opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.18, ease: "easeOut" }}
                        style={{ overflow: "hidden", display: "flex", flexDirection: "column", gap: 4, marginTop: 4 }}
                      >
                        {SIDEBAR_ITEMS.filter((i) => productionKeys.has(i.key)).map((child) => {
                          const isActive = pathname === child.href;
                          return (
                            <Link
                              key={child.key}
                              href={child.href}
                              aria-current={isActive ? "page" : undefined}
                              title={!isExpanded ? child.label : undefined}
                              style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: isExpanded ? "flex-start" : "center",
                                gap: 12,
                                padding: isExpanded ? "9px 12px" : "10px",
                                borderRadius: 8,
                                background: isActive ? "var(--bg-active)" : "transparent",
                                color: isActive ? "var(--text-primary)" : "var(--text-secondary)",
                                textDecoration: "none",
                                transition: "background 0.15s ease, color 0.15s ease",
                                cursor: "pointer",
                                width: isExpanded ? "calc(100% - 14px)" : 44,
                                minHeight: 44,
                                marginLeft: isExpanded ? 14 : 0,
                              }}
                              onMouseEnter={(e) => {
                                if (!isActive) {
                                  e.currentTarget.style.background = "var(--bg-hover)";
                                  e.currentTarget.style.color = "var(--text-primary)";
                                }
                              }}
                              onMouseLeave={(e) => {
                                if (!isActive) {
                                  e.currentTarget.style.background = "transparent";
                                  e.currentTarget.style.color = "var(--text-secondary)";
                                }
                              }}
                            >
                              <FontAwesomeIcon icon={child.icon} className="h-4 w-4" style={{ flexShrink: 0 }} />
                              <AnimatePresence initial={false}>
                                {isExpanded && (
                                  <motion.span
                                    key="label"
                                    initial={{ opacity: 0, x: -6 }}
                                    animate={{ opacity: 1, x: 0 }}
                                    exit={{ opacity: 0, x: -6 }}
                                    transition={{ duration: prefersReducedMotion ? 0 : 0.15, ease: "easeOut" }}
                                    style={{ fontSize: 13, fontWeight: 500, whiteSpace: "nowrap" }}
                                  >
                                    {child.label}
                                  </motion.span>
                                )}
                              </AnimatePresence>
                            </Link>
                          );
                        })}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>,
              );

              continue;
            }

            const isActive = pathname === item.href;

            out.push(
              <Link
                key={item.key}
                href={item.href}
                aria-current={isActive ? "page" : undefined}
                title={!isExpanded ? item.label : undefined}
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: isExpanded ? "flex-start" : "center",
                  gap: 12,
                  padding: isExpanded ? "10px 12px" : "10px",
                  borderRadius: 8,
                  background: isActive ? "var(--bg-active)" : "transparent",
                  color: isActive ? "var(--text-primary)" : "var(--text-secondary)",
                  textDecoration: "none",
                  transition: "background 0.15s ease, color 0.15s ease",
                  cursor: "pointer",
                  width: isExpanded ? "100%" : 44,
                  minHeight: 44,
                }}
                onMouseEnter={(e) => {
                  if (!isActive) {
                    e.currentTarget.style.background = "var(--bg-hover)";
                    e.currentTarget.style.color = "var(--text-primary)";
                  }
                }}
                onMouseLeave={(e) => {
                  if (!isActive) {
                    e.currentTarget.style.background = "transparent";
                    e.currentTarget.style.color = "var(--text-secondary)";
                  }
                }}
              >
                <FontAwesomeIcon icon={item.icon} className="h-4 w-4" style={{ flexShrink: 0 }} />
                <AnimatePresence initial={false}>
                  {isExpanded && (
                    <motion.span
                      key="label"
                      initial={{ opacity: 0, x: -6 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: -6 }}
                      transition={{ duration: prefersReducedMotion ? 0 : 0.15, ease: "easeOut" }}
                      style={{ fontSize: 13, fontWeight: 500, whiteSpace: "nowrap" }}
                    >
                      {item.label}
                    </motion.span>
                  )}
                </AnimatePresence>
              </Link>,
            );
          }

          return out;
        })()}
      </nav>
    </motion.aside>
  );
}
