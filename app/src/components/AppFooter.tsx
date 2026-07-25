import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SetupGuideModal } from "./SetupGuideModal";

const BADGES: { src: string; alt: string; href?: string }[] = [
  { src: "/footer/bestviewedopen.gif", alt: "Best viewed with open eyes" },
  { src: "/footer/bestvw.gif", alt: "This page is best viewed with: a computer and a monitor" },
  { src: "/footer/budgie.gif", alt: "Best viewed with a budgie" },
  { src: "/footer/discord3.gif", alt: "Join my Discord", href: "https://discord.gg/N64qCEqcCT" },
  { src: "/footer/mysql5.gif", alt: "Powered by MySQL" },
  { src: "/footer/mysqla.gif", alt: "Powered by MySQL" },
];

export function AppFooter() {
  const [showGuide, setShowGuide] = useState(false);

  return (
    <>
      <footer className="flex shrink-0 flex-wrap items-center justify-between gap-6 border-t border-border px-8 py-5">
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap items-center gap-4">
            {BADGES.map((badge) =>
              badge.href ? (
                <button
                  key={badge.src}
                  onClick={() => openUrl(badge.href!)}
                  title={badge.alt}
                  className="block cursor-pointer rounded border-0 bg-transparent p-0 opacity-90 transition-all hover:scale-105 hover:opacity-100 active:scale-95"
                >
                  <img src={badge.src} alt={badge.alt} width={88} height={31} />
                </button>
              ) : (
                <img
                  key={badge.src}
                  src={badge.src}
                  alt={badge.alt}
                  title={badge.alt}
                  width={88}
                  height={31}
                  className="opacity-90"
                />
              ),
            )}
          </div>
          <p className="text-xs text-muted">
            the SQL ones are making fun of Demonware, the rest are just for fun. join the Discord
            by clicking the image
          </p>
        </div>

        <button
          onClick={() => setShowGuide(true)}
          className="shrink-0 rounded-lg border border-border px-4 py-2 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.97]"
        >
          Setup &amp; Troubleshooting
        </button>
      </footer>

      {showGuide && <SetupGuideModal onClose={() => setShowGuide(false)} />}
    </>
  );
}
