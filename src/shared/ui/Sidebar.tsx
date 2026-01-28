"use client";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Button } from "@heroui/react";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import Link from "next/link";
import { Fragment, useMemo } from "react";
import { SIDEBAR_ITEMS } from "@/shared/constants/sidebar";
import { useSidebar } from "@/shared/contexts/sidebar";
import { VisorLogo } from "@/shared/ui/VisorLogo";

export function Sidebar() {
  const { isExpanded, width } = useSidebar();
  const prefersReducedMotion = useReducedMotion();

  const containerStyle = useMemo(
    () => ({
      background: "var(--dark-bg-sidebar)",
      borderRight: "1px solid var(--dark-divider)",
      color: "var(--dark-text-secondary)",
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
        <VisorLogo variant="white" size={32} />
        <AnimatePresence initial={false}>
          {isExpanded && (
            <motion.span
              key="brand"
              initial={{ opacity: 0, x: -8 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -8 }}
              transition={{ duration: prefersReducedMotion ? 0 : 0.18, ease: "easeOut" }}
              style={{ fontWeight: 700, fontSize: 15, color: "var(--dark-text-secondary)" }}
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
          gap: 8,
          alignItems: isExpanded ? "stretch" : "center",
        }}
      >
        {SIDEBAR_ITEMS.map((item) => (
          <Fragment key={item.key}>
            {isExpanded ? (
              <Button
                as={Link}
                href="#"
                fullWidth
                variant="light"
                radius="md"
                className="hover:bg-(--dark-bg-hover)"
                style={{
                  justifyContent: "flex-start",
                  color: "var(--dark-text-secondary)",
                  padding: "12px 12px",
                  borderRadius: 12,
                }}
                disableRipple
              >
                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  <FontAwesomeIcon icon={item.icon} className="h-4 w-4" />
                  <AnimatePresence initial={false}>
                    <motion.span
                      key="label"
                      initial={{ opacity: 0, x: -6 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: -6 }}
                      transition={{ duration: prefersReducedMotion ? 0 : 0.18, ease: "easeOut" }}
                      style={{ fontSize: 13, fontWeight: 600, whiteSpace: "nowrap" }}
                    >
                      {item.label}
                    </motion.span>
                  </AnimatePresence>
                </div>
              </Button>
            ) : (
              <Button
                as={Link}
                href="#"
                isIconOnly
                variant="light"
                radius="md"
                aria-label={item.label}
                className="hover:bg-(--dark-bg-hover)"
                style={{
                  width: 44,
                  height: 44,
                  marginLeft: "auto",
                  marginRight: "auto",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  color: "var(--dark-text-secondary)",
                  borderRadius: 12,
                }}
                disableRipple
              >
                <FontAwesomeIcon icon={item.icon} className="h-4 w-4" />
              </Button>
            )}
          </Fragment>
        ))}
      </nav>
    </motion.aside>
  );
}
