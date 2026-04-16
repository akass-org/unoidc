import { Link } from "react-router-dom";
import type { LucideIcon } from "lucide-react";

interface PortalSwitchButtonProps {
  to: string;
  label: string;
  icon: LucideIcon;
}

export function PortalSwitchButton({ to, label, icon: Icon }: PortalSwitchButtonProps) {
  return (
    <Link
      to={to}
      className="inline-flex h-8 min-w-[84px] items-center justify-center gap-1.5 rounded-lg border border-gray-200 bg-gray-50 px-2.5 text-xs text-gray-600 transition-colors hover:border-gray-300 hover:text-gray-800 dark:border-white/[0.08] dark:bg-white/[0.03] dark:text-gray-300 dark:hover:border-white/[0.16] dark:hover:text-white"
    >
      <Icon className="w-3.5 h-3.5" />
      {label}
    </Link>
  );
}
