import { EmblemCard } from "./EmblemCard";
import { IconDownload, IconInbox, IconRefresh, IconTrash } from "./icons";
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
}: Props) {
  const isSelected = (e: EmblemInfo) =>
    selected != null && selected.group === e.group && selected.slot === e.slot;
  const allChecked = emblems.length > 0 && emblems.every((e) => checkedIds.has(e.id));

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <div className="shrink-0">
        <SectionHeading
          title="Your captured emblems"
          action={
            selectMode ? (
              <div className="flex items-center gap-2">
                <button
                  onClick={onToggleSelectAll}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-colors hover:border-fg hover:bg-surface"
                >
                  {allChecked ? "Deselect all" : "Select all"}
                </button>
                <button
                  onClick={onBulkDelete}
                  disabled={checkedIds.size === 0}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold text-red-500 transition-colors hover:border-red-500 hover:bg-surface disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:border-border"
                >
                  <IconTrash size={14} />
                  Delete {checkedIds.size > 0 ? `(${checkedIds.size})` : ""}
                </button>
                <button
                  onClick={onToggleSelectMode}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-colors hover:border-fg hover:bg-surface"
                >
                  Cancel
                </button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <button
                  onClick={onImport}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-colors hover:border-fg hover:bg-surface"
                >
                  <IconDownload size={14} />
                  Import
                </button>
                <button
                  onClick={onRefresh}
                  className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-colors hover:border-fg hover:bg-surface"
                >
                  <IconRefresh size={14} />
                  Refresh
                </button>
                {emblems.length > 0 && (
                  <button
                    onClick={onToggleSelectMode}
                    className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-colors hover:border-fg hover:bg-surface"
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
        <div className="mt-6 min-h-0 flex-1">
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
        <div className="scroll-thin mt-6 min-h-0 flex-1 overflow-y-auto">
          <div className="grid grid-cols-2 gap-4 pb-2 sm:grid-cols-3 lg:grid-cols-4">
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
    </section>
  );
}
