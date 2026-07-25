import type { ReactNode } from "react";
import { IconInfo } from "../icons";

interface NoticeItem {
  title: string;
  children: ReactNode;
}

interface Props {
  heading: string;
  items: NoticeItem[];
}

export function NoticeList({ heading, items }: Props) {
  return (
    <div className="mt-4">
      <h3 className="flex items-center gap-2 text-sm font-bold tracking-tight text-muted">
        <IconInfo size={15} />
        {heading}
      </h3>
      <ul className="mt-1.5 space-y-1 text-sm text-muted">
        {items.map((item) => (
          <li key={item.title}>
            <strong className="text-fg">{item.title}</strong> {item.children}
          </li>
        ))}
      </ul>
    </div>
  );
}
