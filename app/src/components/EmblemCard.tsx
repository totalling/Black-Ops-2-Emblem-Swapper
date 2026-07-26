import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { EmblemInfo } from "../lib/types";
import { IconCheck, IconExport, IconInfo, IconTrash } from "./icons";

function fmtDate(s: string): string {
  if (!s) return "";
  return s.slice(0, 16).replace(" ", " · ");
}

interface Props {
  emblem: EmblemInfo;
  selected: boolean;
  onSelect: () => void;
  onLabelChange: (label: string) => void;
  onExport: () => void;
  onDelete: () => void;
  selectMode: boolean;
  checked: boolean;
  onToggleCheck: () => void;
}

type ThumbState = { status: "loading" } | { status: "ready"; url: string } | { status: "error" };

export function EmblemCard({
  emblem,
  selected,
  onSelect,
  onLabelChange,
  onExport,
  onDelete,
  selectMode,
  checked,
  onToggleCheck,
}: Props) {
  const [thumb, setThumb] = useState<ThumbState>({ status: "loading" });
  const [label, setLabel] = useState(emblem.label);

  useEffect(() => {
    setLabel(emblem.label);
  }, [emblem.label]);

  useEffect(() => {
    let cancelled = false;
    setThumb({ status: "loading" });
    api
      .renderEmblemPng(emblem.group, emblem.slot)
      .then((url) => {
        if (!cancelled) setThumb({ status: "ready", url });
      })
      .catch(() => {
        if (!cancelled) setThumb({ status: "error" });
      });
    return () => {
      cancelled = true;
    };
  }, [emblem.group, emblem.slot]);

  const placeholder = label || `Emblem from ${fmtDate(emblem.captured_at)}`;

  return (
    <div
      onClick={selectMode ? onToggleCheck : onSelect}
      className={`group relative flex cursor-pointer flex-col gap-2 rounded-xl border p-3 transition-all active:scale-[0.98] ${
        selectMode
          ? checked
            ? "border-fg shadow-sm"
            : "border-border hover:border-fg/60 hover:bg-surface/50"
          : selected
            ? "border-fg shadow-sm"
            : "border-border hover:border-fg/60 hover:bg-surface/50"
      }`}
    >
      {selectMode ? (
        <div
          className={`absolute right-5 top-5 z-10 flex h-6 w-6 items-center justify-center rounded-lg border transition-colors ${
            checked ? "border-fg bg-fg text-bg" : "border-border bg-bg text-transparent"
          }`}
        >
          <IconCheck size={14} />
        </div>
      ) : (
        selected && (
          <div className="animate-fade-in absolute right-5 top-5 z-10 flex h-6 w-6 items-center justify-center rounded-lg border border-fg bg-fg text-bg">
            <IconCheck size={14} />
          </div>
        )
      )}

      {!selectMode && (
        <div className="absolute left-5 top-5 z-10 flex items-center gap-1.5 opacity-0 transition-opacity duration-200 group-hover:opacity-100">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onExport();
            }}
            title="Save a copy of this emblem to a file"
            className="flex h-6 w-6 items-center justify-center rounded-lg border border-border bg-bg text-muted transition-all hover:text-fg active:scale-90"
          >
            <IconExport size={12} />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onDelete();
            }}
            title="Delete this emblem"
            className="flex h-6 w-6 items-center justify-center rounded-lg border border-border bg-bg text-muted transition-all hover:border-red-500 hover:text-red-500 active:scale-90"
          >
            <IconTrash size={12} />
          </button>
        </div>
      )}

      <div className="flex aspect-square items-center justify-center overflow-hidden rounded-lg bg-surface">
        {thumb.status === "ready" && (
          <img src={thumb.url} alt="" className="animate-fade-in h-full w-full object-contain" />
        )}
        {thumb.status === "loading" && <div className="h-8 w-8 animate-pulse bg-border" />}
        {thumb.status === "error" && (
          <div className="flex flex-col items-center gap-1 text-muted" title="Couldn't render this emblem">
            <IconInfo size={18} />
            <span className="text-xs">Render failed</span>
          </div>
        )}
      </div>

      <input
        value={label}
        placeholder={placeholder}
        title={label || placeholder}
        onClick={(e) => e.stopPropagation()}
        onChange={(e) => setLabel(e.target.value)}
        onBlur={() => {
          if (label !== emblem.label) onLabelChange(label);
        }}
        className="w-full truncate border-b border-transparent bg-transparent text-sm font-bold tracking-tight outline-none placeholder:text-muted placeholder:font-normal focus:border-border"
      />
    </div>
  );
}
