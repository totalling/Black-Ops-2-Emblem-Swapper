import { useState } from "react";
import { DuplicatesModal } from "./DuplicatesModal";
import { EmblemCard } from "./EmblemCard";
import { IconArchive, IconCopy, IconDownload, IconInbox, IconRefresh, IconTrash } from "./icons";
import { EmptyState } from "./ui/EmptyState";
import { SectionHeading } from "./ui/SectionHeading";
import type { EmblemInfo, Selection } from "../lib/types";

interface Props {
  emblems: EmblemInfo[];
  selected: Selection | null;
  onSelect: (group: string, slot: number) => void;
  onLabelChange: (group: string, slot: number, label: string) => void;
  onExport: (emblem: EmblemInfo) => void;
  onDelete: (emblem: EmblemInfo) => void;
  onImport: () => void;
  onRefresh: () => void;
  selectMode: boolean;
  checkedIds: Set<string>;
  onToggleSelectMode: () => void;
  onToggleCheck: (id: string) => void;
  onToggleSelectAll: () => void;
  onBulkDelete: () => void;
  onBackup: () => void;
  onRestore: () => void;
}

export function EmblemGrid({
  emblems,
  selected,
  onSelect,
  onLabelChange,
  onExport,
  onDelete,
  onImport,
  onRefresh,
  selectMode,
  checkedIds,
  onToggleSelectMode,
  onToggleCheck,
  onToggleSelectAll,
  onBulkDelete,
  onBackup,
  onRestore,
}: Props) {
  const isSelected = (e: EmblemInfo) =>
    selected != null && selected.group === e.group && selected.slot === e.slot;
  const allChecked = emblems.length > 0 && emblems.every((e) => checkedIds.has(e.id));
  const [showDuplicates, setShowDuplicates] = useState(false);

  return (
    <section className="flex flex-col lg:min-h-0 lg:flex-1">
      <div className="shrink-0">
        <SectionHeading
          title="Your captured emblems"
          action={
            selectMode ? (
              <div className="flex flex-wrap items-center gap-2">
                <button
                  onClick={onToggleSelectAll}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                >
                  {allChecked ? "Deselect all" : "Select all"}
                </button>
                <button
                  onClick={onBulkDelete}
                  disabled={checkedIds.size === 0}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold text-red-500 transition-all hover:border-red-500 hover:bg-surface active:scale-[0.96] disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:border-border disabled:active:scale-100"
                >
                  <IconTrash size={14} />
                  Delete {checkedIds.size > 0 ? `(${checkedIds.size})` : ""}
                </button>
                <button
                  onClick={onToggleSelectMode}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                >
                  Cancel
                </button>
              </div>
            ) : (
              <div className="flex flex-wrap items-center gap-2">
                <button
                  onClick={onImport}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                >
                  <IconDownload size={14} />
                  Import
                </button>
                <button
                  onClick={onRefresh}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                >
                  <IconRefresh size={14} />
                  Refresh
                </button>
                <button
                  onClick={onRestore}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                >
                  <IconArchive size={14} />
                  Restore Backup
                </button>
                {emblems.length > 0 && (
                  <button
                    onClick={onBackup}
                    className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                  >
                    <IconArchive size={14} />
                    Backup
                  </button>
                )}
                {emblems.length > 0 && (
                  <button
                    onClick={() => setShowDuplicates(true)}
                    className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                  >
                    <IconCopy size={14} />
                    Find Duplicates
                  </button>
                )}
                {emblems.length > 0 && (
                  <button
                    onClick={onToggleSelectMode}
                    className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
                  >
                    <IconTrash size={14} />
                    Select
                  </button>
                )}
              </div>
            )
          }
        />

        <p className="mt-2 text-sm text-muted">
          Click an emblem to select it. Switch to <strong className="text-fg">Show</strong> and
          open your emblem editor: this is what loads there, ready to save. Export one to keep a
          backup, or import a <code className="text-fg">.bin</code> file someone shared with you.
        </p>
      </div>

      {emblems.length === 0 ? (
        <div className="mt-6 lg:min-h-0 lg:flex-1">
          <EmptyState
            icon={<IconInbox size={20} />}
            title="Nothing captured yet"
            description={
              <>
                Switch to <strong className="text-fg">Capture</strong> mode and open a
                player&apos;s profile on your PS5 to save your first one, or{" "}
                <strong className="text-fg">Import</strong> one you already have.
              </>
            }
          />
        </div>
      ) : (
        <div className="scroll-thin mt-6 lg:min-h-0 lg:flex-1 lg:overflow-y-auto">
          <div className="animate-fade-in grid grid-cols-2 gap-4 pb-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
            {emblems.map((e) => (
              <EmblemCard
                key={e.id}
                emblem={e}
                selected={isSelected(e)}
                onSelect={() => onSelect(e.group, e.slot)}
                onLabelChange={(label) => onLabelChange(e.group, e.slot, label)}
                onExport={() => onExport(e)}
                onDelete={() => onDelete(e)}
                selectMode={selectMode}
                checked={checkedIds.has(e.id)}
                onToggleCheck={() => onToggleCheck(e.id)}
              />
            ))}
          </div>
        </div>
      )}

      {showDuplicates && (
        <DuplicatesModal
          onClose={() => setShowDuplicates(false)}
          onResolved={onRefresh}
        />
      )}
    </section>
  );
}
