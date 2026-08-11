interface Props {
  message: string;
  onDismiss: () => void;
}

export function ErrorToast({ message, onDismiss }: Props) {
  return (
    <div className="animate-toast-in fixed bottom-[calc(1.5rem+env(safe-area-inset-bottom))] left-1/2 z-50 flex max-w-md items-start gap-3 rounded-xl border border-fg bg-bg p-4 shadow-lg">
      <p className="text-sm">
        <strong className="text-fg">Something went wrong.</strong>{" "}
        <span className="text-muted">{message}</span>
      </p>
      <button
        onClick={onDismiss}
        className="shrink-0 rounded-lg border border-border px-2 py-1 text-xs font-bold transition-all hover:border-fg hover:bg-surface active:scale-[0.96]"
      >
        Dismiss
      </button>
    </div>
  );
}
