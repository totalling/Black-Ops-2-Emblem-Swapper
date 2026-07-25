import type { SVGProps } from "react";

interface IconProps extends SVGProps<SVGSVGElement> {
  size?: number;
}

function Base({ size = 20, children, ...rest }: IconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="square"
      strokeLinejoin="miter"
      shapeRendering="crispEdges"
      className="pixel-icon"
      aria-hidden="true"
      {...rest}
    >
      {children}
    </svg>
  );
}

export function IconPower(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M12 3 L12 11" />
      <path d="M16 7 L19 10 L19 16 L16 19 L8 19 L5 16 L5 10 L8 7" />
    </Base>
  );
}

export function IconCamera(props: IconProps) {
  return (
    <Base {...props}>
      <rect x={3} y={7} width={18} height={12} />
      <rect x={8} y={4} width={8} height={4} />
      <rect x={9} y={11} width={6} height={6} />
      <rect x={17} y={9} width={2} height={2} fill="currentColor" stroke="none" />
    </Base>
  );
}

export function IconShare(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M12 4 L12 15" />
      <path d="M8 8 L12 4 L16 8" />
      <path d="M4 14 L4 19 L20 19 L20 14" />
    </Base>
  );
}

export function IconCheck(props: IconProps) {
  return (
    <Base strokeWidth={2.5} {...props}>
      <path d="M4 12 L9 17 L20 5" />
    </Base>
  );
}

export function IconRefresh(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M5 8 L16 8 L13 5" />
      <path d="M16 8 L13 11" />
      <path d="M19 16 L8 16 L11 19" />
      <path d="M8 16 L11 13" />
    </Base>
  );
}

export function IconInfo(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M16 7 L19 10 L19 16 L16 19 L8 19 L5 16 L5 10 L8 7 Z" />
      <rect x={11} y={8} width={2} height={2} fill="currentColor" stroke="none" />
      <path d="M12 11.5 L12 16" />
    </Base>
  );
}

export function IconInbox(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M4 8 L4 18 L20 18 L20 8" />
      <path d="M4 8 L9 8 L9 12 L15 12 L15 8 L20 8" />
    </Base>
  );
}

export function IconDownload(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M12 4 L12 15" />
      <path d="M8 11 L12 15 L16 11" />
      <path d="M4 19 L20 19" />
    </Base>
  );
}

export function IconExport(props: IconProps) {
  return (
    <Base {...props}>
      <rect x={4} y={4} width={11} height={11} />
      <path d="M13 15 L20 15 L20 8" />
      <path d="M20 8 L13 15" />
    </Base>
  );
}

export function IconCopy(props: IconProps) {
  return (
    <Base {...props}>
      <rect x={4} y={8} width={12} height={12} />
      <path d="M8 8 L8 4 L20 4 L20 16 L16 16" />
    </Base>
  );
}

export function IconPulse(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M3 13 L8 13 L10 7 L14 19 L16 13 L21 13" />
    </Base>
  );
}

export function IconArchive(props: IconProps) {
  return (
    <Base {...props}>
      <rect x={4} y={4} width={16} height={5} />
      <path d="M5 9 L5 20 L19 20 L19 9" />
      <path d="M10 13 L14 13" />
    </Base>
  );
}

export function IconClose(props: IconProps) {
  return (
    <Base strokeWidth={2.5} {...props}>
      <path d="M5 5 L19 19" />
      <path d="M19 5 L5 19" />
    </Base>
  );
}

export function IconWrench(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M14 4 L20 10 L17 13 L11 7 Z" />
      <path d="M13 11 L4 20" />
    </Base>
  );
}

export function IconTrash(props: IconProps) {
  return (
    <Base {...props}>
      <path d="M5 7 L19 7" />
      <path d="M9 7 L9 4 L15 4 L15 7" />
      <path d="M7 7 L8 20 L16 20 L17 7" />
      <path d="M10 10 L10 17" />
      <path d="M14 10 L14 17" />
    </Base>
  );
}

