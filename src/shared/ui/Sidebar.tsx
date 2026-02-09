"use client";

import React from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faChevronDown } from "@fortawesome/free-solid-svg-icons";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { usePathname, useRouter } from "next/navigation";
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
  const router = useRouter();
  const [openGroups, setOpenGroups] = useState<Record<string, boolean>>({
    production: true,
    "data-exchange": true,
    "data-exchange-logitron": true,
  });

  useEffect(() => {
    if (!isExpanded) {
      // Use timeout to avoid synchronoussetState warning and potential layout thrashing
      const timer = setTimeout(() => setOpenGroups({}), 0);
      return () => clearTimeout(timer);
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

  return (
    <motion.aside
      initial={false}
      animate={{ width }}
      transition={
        prefersReducedMotion
          ? { duration: 0 }
          : { type: "spring", stiffness: 320, damping: 34 }
      }
      className="flex flex-col gap-2.5 overflow-hidden whitespace-nowrap border-r border-(--border-default) bg-(--bg-secondary) text-(--text-secondary)"
      style={{
        padding: isExpanded ? "12px 10px" : "12px 8px",
        willChange: "width",
      }}
    >
      <div
        className={
          isExpanded
            ? "flex items-center justify-start gap-2.5 px-2.5 py-1.5"
            : "flex items-center justify-center gap-2.5 px-1.5 py-1.5"
        }
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
              className="text-[15px] font-bold text-(--text-primary)"
            >
              Visor
            </motion.span>
          )}
        </AnimatePresence>
      </div>

      <nav className={`flex flex-col gap-1 ${isExpanded ? "items-stretch" : "items-center"}`}>
        {SIDEBAR_MENU.map((item) => {
          const renderItem = (it: SidebarItem, depth: number): React.ReactElement => {
            const hasChildren = !!it.children?.length;
            const groupOpen = !!openGroups[it.key];
            const active = isItemActive(it);
            const indent = isExpanded ? depth * 14 : 0;

            const baseClasses =
              "flex w-full min-h-[44px] items-center gap-2 rounded-lg border-none transition-colors duration-150 ease-out cursor-pointer";
            const justification = isExpanded ? "justify-between" : "justify-center";
            const padding = isExpanded ? "px-3 py-2.5" : "p-2.5";
            const activeClasses = active
              ? "bg-(--bg-active) text-(--text-primary)"
              : "bg-transparent text-(--text-secondary) group-hover:bg-(--bg-hover) group-hover:text-(--text-primary) hover:bg-(--bg-hover) hover:text-(--text-primary)";

            if (hasChildren) {
              return (
                <div key={it.key} className="w-full">
                  <button
                    type="button"
                    title={!isExpanded ? it.label : undefined}
                    onClick={() => setOpenGroups((prev) => ({ ...prev, [it.key]: !prev[it.key] }))}
                    className={`${baseClasses} ${justification} ${padding} ${activeClasses}`}
                    style={{ paddingLeft: isExpanded ? 12 + indent : 0 }}
                  >
                    <div className="flex min-w-0 items-center gap-2.5">
                      <FontAwesomeIcon icon={it.icon} className="h-4 w-4 shrink-0" />
                      <AnimatePresence initial={false}>
                        {isExpanded && (
                          <motion.span
                            key={`${it.key}-label`}
                            initial={{ opacity: 0, x: -6 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -6 }}
                            transition={{
                              duration: prefersReducedMotion ? 0 : 0.15,
                              ease: "easeOut",
                            }}
                            className="overflow-hidden text-ellipsis whitespace-nowrap text-[13px] font-semibold"
                          >
                            {it.label}
                          </motion.span>
                        )}
                      </AnimatePresence>
                    </div>
                    {isExpanded && (
                      <motion.span
                        animate={{ rotate: groupOpen ? 0 : -90 }}
                        transition={
                          prefersReducedMotion ? { duration: 0 } : { duration: 0.18, ease: "easeOut" }
                        }
                        className="flex w-4 items-center justify-center"
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
                        transition={
                          prefersReducedMotion ? { duration: 0 } : { duration: 0.18, ease: "easeOut" }
                        }
                        className="mt-1 flex flex-col gap-1 overflow-hidden"
                      >
                        {it.children!.map((c) => renderItem(c, depth + 1))}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              );
            }

            const isActive = !!it.href && pathname === it.href;
            const itemActiveClasses = isActive
              ? "bg-(--bg-active) text-(--text-primary)"
              : "bg-transparent text-(--text-secondary) hover:bg-(--bg-hover) hover:text-(--text-primary)";

            return (
              <div
                key={it.key}
                role="button"
                tabIndex={0}
                onClick={() => {
                  if (it.href) router.push(it.href);
                }}
                onKeyDown={(e) => {
                  if ((e.key === "Enter" || e.key === " ") && it.href) {
                    e.preventDefault();
                    router.push(it.href);
                  }
                }}
                className={`group ${baseClasses} ${isExpanded ? "justify-start" : "justify-center"} ${padding} ${itemActiveClasses}`}
                aria-current={isActive ? "page" : undefined}
                title={!isExpanded ? it.label : undefined}
                style={{
                  width: isExpanded ? "100%" : 44,
                  minHeight: 44,
                  paddingLeft: isExpanded ? 12 + indent : 0,
                }}
              >
                <FontAwesomeIcon icon={it.icon} className="h-4 w-4 shrink-0" />
                <AnimatePresence initial={false}>
                  {isExpanded && (
                    <motion.span
                      key={`${it.key}-label`}
                      initial={{ opacity: 0, x: -6 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: -6 }}
                      transition={{
                        duration: prefersReducedMotion ? 0 : 0.15,
                        ease: "easeOut",
                      }}
                      className="whitespace-nowrap text-[13px] font-medium"
                    >
                      {it.label}
                    </motion.span>
                  )}
                </AnimatePresence>
              </div>
            );
          };

          return renderItem(item, 0);
        })}
      </nav>
    </motion.aside>
  );
}
