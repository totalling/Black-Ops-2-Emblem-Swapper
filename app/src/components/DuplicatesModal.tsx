import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { EmblemInfo } from "../lib/types";
import { IconCopy, IconInfo, IconTrash } from "./icons";

interface Props {
  onClose: () => void;
  onResolved: () => void;
}

type ThumbState = { status: "loading" } | { status: "ready"; url: string } | { status: "error" };

function DupThumb({ emblem }: { emblem: EmblemInfo }) {
  const [thumb, setThumb] = useState<ThumbState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
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

  return (
    <div className="flex aspect-square w-24 items-center justify-center overflow-hidden rounded-lg bg-surface">
      {thumb.status === "ready" && <img src={thumb.url} alt="" className="h-full w-full object-contain" />}
      {thumb.status === "loading" && <div className="h-6 w-6 animate-pulse bg-border" />}
      {thumb.status === "error" && <IconInfo size={16} />}
    </div>
  );
}

export function DuplicatesModal({ onClose, onResolved }: Props) {
  const [loading, setLoading] = useState(true);
  const [groups, setGroups] = useState<EmblemInfo[][]>([]);
  const [toDelete, setToDelete] = useState<Set<string>>(new Set());
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .findDuplicateEmblems()
      .then((found) => {
        setGroups(found);
        const preChecked = new Set<string>();
        found.forEach((group) => group.slice(1).forEach((e) => preChecked.add(e.id)));
        setToDelete(preChecked);
        setLoading(false);
      })
      .catch((e) => {
        setError(String(e));
        setLoading(false);
      });
  }, []);

  const toggle = (id: string) =>
    setToDelete((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });

  const handleDelete = async () => {
    if (toDelete.size === 0) return;
    setDeleting(true);
    setError(null);
    try {
      const items = [...toDelete].map((id) => {
        const [group, slot] = id.split(":");
        return { group, slot: Number(slot) };
      });
      await api.deleteEmblems(items);
      onResolved();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div
      className="animate-overlay-in fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-8 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="animate-modal-in max-h-[85vh] w-full max-w-2xl overflow-y-auto rounded-2xl border border-border bg-bg p-6 shadow-2xl"
      >
        <div className="mb-5 flex items-center justify-between">
          <h3 className="flex items-center gap-2 text-lg font-bold tracking-tight">
            <IconCopy size={18} />
            Duplicate emblems
          </h3>
          <button
            onClick={onClose}
            className="rounded-lg border border-border px-2 py-1 text-sm transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
          >
            Close
          </button>
        </div>

        {loading && <p className="text-sm text-muted">Scanning your library…</p>}

        {!loading && groups.length === 0 && (
          <p className="animate-fade-in text-sm text-muted">
            No duplicates found. Every capture is unique.
          </p>
        )}

        {!loading && groups.length > 0 && (
          <div className="animate-fade-in space-y-5">
            <p className="text-sm text-muted">
              These are byte-for-byte identical captures. The oldest one in each group is kept by
              default, check whichever copies you want to delete.
            </p>

            {groups.map((group, i) => (
              <div key={i} className="rounded-xl border border-border p-3">
                <p className="mb-2 text-xs font-bold uppercase tracking-wider text-muted">
                  {group.length} identical copies
                </p>
                <div className="flex flex-wrap gap-3">
                  {group.map((emblem) => (
                    <label
                      key={emblem.id}
                      className="flex w-24 cursor-pointer flex-col items-center gap-1.5"
                    >
                      <DupThumb emblem={emblem} />
                      <span className="w-full truncate text-center text-xs text-muted">
                        {emblem.label || emblem.captured_at}
                      </span>
                      <input
                        type="checkbox"
                        checked={toDelete.has(emblem.id)}
                        onChange={() => toggle(emblem.id)}
                      />
                    </label>
                  ))}
                </div>
              </div>
            ))}

            <button
              onClick={handleDelete}
              disabled={deleting || toDelete.size === 0}
              className="flex w-full items-center justify-center gap-2 rounded-lg border border-border px-3 py-2 text-sm font-bold text-red-500 transition-all hover:border-red-500 hover:bg-surface active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-40 disabled:active:scale-100"
            >
              <IconTrash size={14} />
              {deleting ? "Deleting…" : `Delete ${toDelete.size} selected`}
            </button>
          </div>
        )}

        {error && (
          <p className="mt-3 rounded-lg border border-danger/40 bg-danger/10 p-3 text-sm text-danger">
            {error}
          </p>
        )}
      </div>
    </div>
  );
}
