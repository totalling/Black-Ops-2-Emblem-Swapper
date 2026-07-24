interface Props {
  message: string;
  onDismiss: () => void;
}

export function ErrorToast({ message, onDismiss }: Props) {
  return (
    <div className="fixed bottom-6 left-1/2 z-50 flex max-w-md -translate-x-1/2 items-start gap-3 rounded-xl border border-fg bg-bg p-4 shadow-sm">
      <p className="text-sm">
        <strong className="text-fg">Something went wrong.</strong>{" "}
        <span className="text-muted">{message}</span>
      </p>
      <button
        onClick={onDismiss}
        className="shrink-0 rounded-lg border border-border px-2 py-1 text-xs font-bold hover:border-fg hover:bg-surface"
      >
        Dismiss
      </button>
    </div>
  );
}
