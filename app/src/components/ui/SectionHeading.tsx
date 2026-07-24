import type { ReactNode } from "react";

interface Props {
  title: string;
  action?: ReactNode;
}

export function SectionHeading({ title, action }: Props) {
  return (
    <div className="flex items-center justify-between">
      <h2 className="text-base font-bold tracking-tight">{title}</h2>
      {action}
    </div>
  );
}
