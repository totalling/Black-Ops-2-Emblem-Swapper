import { IconCamera, IconPower, IconShare } from "./icons";
import { NoticeList } from "./ui/NoticeList";
import { SectionHeading } from "./ui/SectionHeading";
import type { PublicMode } from "../lib/types";

const MODES: {
  mode: PublicMode;
  name: string;
  desc: string;
  icon: (props: { size?: number }) => React.ReactElement;
}[] = [
  {
    mode: "off",
    name: "Off",
    desc: "Nothing is touched, your PS5 browses normally",
    icon: IconPower,
  },
  {
    mode: "capture",
    name: "Capture",
    desc: "Saves the emblem of any player whose profile you open",
    icon: IconCamera,
  },
  {
    mode: "inject",
    name: "Show",
    desc: "Loads your selected emblem into your emblem editor",
    icon: IconShare,
  },
];

const MODE_HINTS: Record<PublicMode, string> = {
  off: "Nothing is captured or changed while this is on.",
  capture:
    "Open a player's profile or channel on your PS5. Their emblem saves below automatically.",
  inject:
    "Open your own emblem editor on the PS5. The emblem you selected below loads there instead of your usual one.",
};

interface Props {
  mode: PublicMode;
  onChange: (mode: PublicMode) => void;
}

export function ModeSwitcher({ mode, onChange }: Props) {
  return (
    <section>
      <SectionHeading title="Mode" />

      <div className="mt-3 grid grid-cols-1 gap-2.5 sm:grid-cols-3">
        {MODES.map(({ mode: m, name, desc, icon: Icon }) => {
          const active = m === mode;
          return (
            <button
              key={m}
              onClick={() => onChange(m)}
              className={`group flex flex-col items-start gap-2 rounded-xl border p-3.5 text-left transition-all ${
                active
                  ? "border-fg bg-fg text-bg shadow-sm"
                  : "border-border hover:border-fg/40 hover:bg-surface"
              }`}
            >
              <div
                className={`flex h-8 w-8 items-center justify-center rounded-lg transition-colors ${
                  active ? "bg-bg/15" : "bg-surface group-hover:bg-border/70"
                }`}
              >
                <Icon size={16} />
              </div>
              <span className="font-bold tracking-tight">{name}</span>
              <span className={`text-sm ${active ? "text-bg/80" : "text-muted"}`}>{desc}</span>
            </button>
          );
        })}
      </div>

      <p className="mt-3 text-sm text-muted">{MODE_HINTS[mode]}</p>

      <NoticeList
        heading="Good to know"
        items={[
          {
            title: "One emblem per game session.",
            children:
              "Black Ops II only checks for a new emblem once per launch. To load a different one, restart the game or switch between Multiplayer and Zombies.",
          },
          {
            title: "Unlocked content only.",
            children:
              "You can't save an emblem that uses a shape or item you haven't unlocked yourself. That's a rule from the game, not this tool.",
          },
        ]}
      />
    </section>
  );
}
