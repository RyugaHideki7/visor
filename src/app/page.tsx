"use client";

import { Button, Card, CardBody, Divider } from "@heroui/react";
import { VisorLogo } from "@/components/VisorLogo";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faArrowUpRightFromSquare } from "@fortawesome/free-solid-svg-icons";

const palette = {
  base: "var(--color-base)",
  surface: "var(--color-surface)",
  slate: "var(--color-slate)",
  accent: "var(--color-accent)",
  navy: "var(--color-navy)",
  amber: "var(--color-amber)",
  text: "var(--color-text)",
  muted: "var(--color-muted)",
};

export default function Home() {
  return (
    <div className="min-h-screen" style={{ background: palette.base, color: palette.text }}>
      <div className="mx-auto flex min-h-screen max-w-5xl flex-col items-center px-6 py-16 sm:px-10">
        <div className="flex w-full flex-col gap-6 rounded-3xl" style={{ background: palette.surface, boxShadow: "0 25px 80px rgba(0,0,0,0.35)" }}>
          <div className="flex flex-col gap-6 p-8 sm:p-10 lg:flex-row lg:items-center lg:gap-8">
            <div className="self-start rounded-3xl" style={{ background: palette.slate, padding: "18px" }}>
              <VisorLogo size={96} />
            </div>
            <div className="flex flex-1 flex-col gap-3">
              <p className="text-xs font-semibold uppercase tracking-[0.14em]" style={{ color: palette.accent }}>
                Visor
              </p>
              <h1 className="text-3xl font-semibold leading-tight sm:text-4xl" style={{ color: palette.text }}>
                A focused, dark-native shell for your Tauri + Next app
              </h1>
              <p className="max-w-3xl text-base leading-7" style={{ color: palette.muted }}>
                Muted, Notion-inspired surfaces with your custom visor mark in color and white variants. HeroUI and FontAwesome are wired and ready.
              </p>
              <div className="flex flex-wrap gap-3 pt-2">
                <Button
                  color="primary"
                  className="text-sm font-medium"
                  style={{ background: palette.accent, color: "white", boxShadow: "0 10px 35px rgba(45,154,166,0.35)" }}
                  endContent={<FontAwesomeIcon icon={faArrowUpRightFromSquare} />}
                >
                  Open docs
                </Button>
                <Button variant="bordered" className="text-sm font-medium" style={{ borderColor: "#2f3744", color: palette.text }}>
                  Launch app
                </Button>
              </div>
            </div>
          </div>

          <Divider className="border-none" style={{ height: 1, background: "#212734" }} />

          <div className="grid gap-4 px-6 pb-8 sm:grid-cols-2 sm:px-8">
            <Card className="border-0 shadow-none" style={{ background: palette.slate, color: palette.text, boxShadow: "0 12px 40px rgba(0,0,0,0.25)" }}>
              <CardBody className="flex items-start gap-3">
                <VisorLogo size={36} />
                <div>
                  <p className="text-sm font-semibold" style={{ color: palette.text }}>
                    Color mark
                  </p>
                  <p className="text-sm" style={{ color: palette.muted }}>
                    Muted teal/navy/gold adapted for dark surfaces.
                  </p>
                </div>
              </CardBody>
            </Card>
            <Card className="border-0 shadow-none" style={{ background: palette.slate, color: palette.text, boxShadow: "0 12px 40px rgba(0,0,0,0.25)" }}>
              <CardBody className="flex items-start gap-3">
                <VisorLogo variant="white" size={36} className="rounded-xl" />
                <div>
                  <p className="text-sm font-semibold" style={{ color: palette.text }}>
                    White mark
                  </p>
                  <p className="text-sm" style={{ color: palette.muted }}>
                    For titlebar/taskbar on dark chrome.
                  </p>
                </div>
              </CardBody>
            </Card>
          </div>

          <Divider className="border-none" style={{ height: 1, background: "#212734" }} />

          <div className="flex flex-col gap-3 px-6 pb-8 sm:px-8">
            <p className="text-sm font-semibold" style={{ color: palette.text }}>
              Palette
            </p>
            <div className="flex flex-wrap gap-3">
              <Swatch label="Base" value={palette.base} />
              <Swatch label="Surface" value={palette.surface} />
              <Swatch label="Slate" value={palette.slate} />
              <Swatch label="Accent" value={palette.accent} />
              <Swatch label="Navy" value={palette.navy} />
              <Swatch label="Amber" value={palette.amber} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Swatch({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center gap-2 rounded-xl px-3 py-2" style={{ background: "#1a202b", color: "#d6d9df" }}>
      <span className="h-8 w-8 rounded-lg" style={{ background: value }} />
      <div className="flex flex-col leading-tight">
        <span className="text-xs font-semibold uppercase tracking-wide">{label}</span>
        <span className="text-xs" style={{ color: "#9aa3b5" }}>{value}</span>
      </div>
    </div>
  );
}
