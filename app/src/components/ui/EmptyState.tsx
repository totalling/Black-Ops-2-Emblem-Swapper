import type { ReactNode } from "react";

interface Props {
  icon: ReactNode;
  title: string;
  description: ReactNode;
}

export function EmptyState({ icon, title, description }: Props) {
  return (
    <div className="mt-6 flex flex-col items-center gap-3 rounded-xl border border-dashed border-border py-14 text-center">
      <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-border text-muted">
        {icon}
      </div>
      <div>
        <p className="font-bold tracking-tight">{title}</p>
        <p className="mt-1 text-sm text-muted">{description}</p>
      </div>
    </div>
  );
}
