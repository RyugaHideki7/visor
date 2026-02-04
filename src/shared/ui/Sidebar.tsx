"use client";

import React from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faChevronDown } from "@fortawesome/free-solid-svg-icons";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useMemo, useState } from "react";
import { SIDEBAR_MENU, type SidebarItem } from "@/shared/constants/sidebar";
import { useSidebar } from "@/shared/contexts/sidebar";
import { useTheme } from "@/shared/contexts/theme";
import { VisorLogo } from "@/shared/ui/VisorLogo";

export function Sidebar() {
  const { isExpanded, width } = useSidebar();
  const { theme } = useTheme();
  const prefersReducedMotion = useReducedMotion();
  const pathname = usePathname();
  const [openGroups, setOpenGroups] = useState<Record<string, boolean>>({
    production: true,
    "data-exchange": true,
    "data-exchange-logitron": true,
  });

  useEffect(() => {
    if (!isExpanded) {
      setOpenGroups({});
    }
  }, [isExpanded]);

  const isItemActive = useMemo(() => {
    const visit = (item: SidebarItem): boolean => {
      if (item.href && pathname === item.href) return true;
      if (item.children) return item.children.some(visit);
      return false;
    };

    return (item: SidebarItem) => visit(item);
  }, [pathname]);

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
        {SIDEBAR_MENU.map((item) => {
          const renderItem = (it: SidebarItem, depth: number): React.ReactElement => {
            const hasChildren = !!it.children?.length;
            const groupOpen = !!openGroups[it.key];
            const active = isItemActive(it);

            const commonStyle = {
              display: "flex",
              alignItems: "center",
              justifyContent: isExpanded ? "space-between" : "center",
              gap: 8,
              padding: isExpanded ? "10px 12px" : "10px",
              borderRadius: 8,
              border: "none",
              width: "100%",
              minHeight: 44,
              background: active ? "var(--bg-active)" : "transparent",
              color: active ? "var(--text-primary)" : "var(--text-secondary)",
              cursor: "pointer",
              transition: "background 0.15s ease, color 0.15s ease",
            } as const;

            const indent = isExpanded ? depth * 14 : 0;

            if (hasChildren) {
              return (
                <div key={it.key} style={{ width: "100%" }}>
                  <button
                    type="button"
                    title={!isExpanded ? it.label : undefined}
                    onClick={() => setOpenGroups((prev) => ({ ...prev, [it.key]: !prev[it.key] }))}
                    style={{ ...commonStyle, paddingLeft: isExpanded ? 12 + indent : 0 }}
                    onMouseEnter={(e) => {
                      if (!active) {
                        e.currentTarget.style.background = "var(--bg-hover)";
                        e.currentTarget.style.color = "var(--text-primary)";
                      }
                    }}
                    onMouseLeave={(e) => {
                      if (!active) {
                        e.currentTarget.style.background = "transparent";
                        e.currentTarget.style.color = "var(--text-secondary)";
                      }
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: 10, minWidth: 0 }}>
                      <FontAwesomeIcon icon={it.icon} className="h-4 w-4" style={{ flexShrink: 0 }} />
                      <AnimatePresence initial={false}>
                        {isExpanded && (
                          <motion.span
                            key={`${it.key}-label`}
                            initial={{ opacity: 0, x: -6 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -6 }}
                            transition={{ duration: prefersReducedMotion ? 0 : 0.15, ease: "easeOut" }}
                            style={{ fontSize: 13, fontWeight: 600, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}
                          >
                            {it.label}
                          </motion.span>
                        )}
                      </AnimatePresence>
                    </div>
                    {isExpanded && (
                      <motion.span
                        animate={{ rotate: groupOpen ? 0 : -90 }}
                        transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.18, ease: "easeOut" }}
                        style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: 16 }}
                      >
                        <FontAwesomeIcon icon={faChevronDown} className="h-3 w-3" />
                      </motion.span>
                    )}
                  </button>

                  <AnimatePresence initial={false}>
                    {isExpanded && groupOpen && (
                      <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: "auto", opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        transition={prefersReducedMotion ? { duration: 0 } : { duration: 0.18, ease: "easeOut" }}
                        style={{ overflow: "hidden", display: "flex", flexDirection: "column", gap: 4, marginTop: 4 }}
                      >
                        {it.children!.map((c) => renderItem(c, depth + 1))}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              );
            }

            const isActive = !!it.href && pathname === it.href;
            return (
              <Link
                key={it.key}
                href={it.href ?? "#"}
                aria-current={isActive ? "page" : undefined}
                title={!isExpanded ? it.label : undefined}
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
                  paddingLeft: isExpanded ? 12 + indent : 0,
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
                <FontAwesomeIcon icon={it.icon} className="h-4 w-4" style={{ flexShrink: 0 }} />
                <AnimatePresence initial={false}>
                  {isExpanded && (
                    <motion.span
                      key={`${it.key}-label`}
                      initial={{ opacity: 0, x: -6 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: -6 }}
                      transition={{ duration: prefersReducedMotion ? 0 : 0.15, ease: "easeOut" }}
                      style={{ fontSize: 13, fontWeight: 500, whiteSpace: "nowrap" }}
                    >
                      {it.label}
                    </motion.span>
                  )}
                </AnimatePresence>
              </Link>
            );
          };

          return renderItem(item, 0);
        })}
      </nav>
    </motion.aside>
  );
}
