import type { ReactNode } from "react";

interface BadgeProps {
  children: ReactNode;
  variant?: "default" | "success" | "warning" | "error" | "info";
  size?: "sm" | "md";
}

export function Badge({ children, variant = "default", size = "sm" }: BadgeProps) {
  const variants = {
    default:
      "bg-gray-100 dark:bg-white/[0.06] text-gray-600 dark:text-gray-400 border-gray-200 dark:border-white/[0.08]",
    success:
      "bg-emerald-50 dark:bg-emerald-500/[0.08] text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-500/[0.16]",
    warning:
      "bg-amber-50 dark:bg-amber-500/[0.08] text-amber-600 dark:text-amber-400 border-amber-200 dark:border-amber-500/[0.16]",
    error:
      "bg-red-50 dark:bg-red-500/[0.08] text-red-600 dark:text-red-400 border-red-200 dark:border-red-500/[0.16]",
    info: "bg-blue-50 dark:bg-blue-500/[0.08] text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-500/[0.16]",
  };

  const sizes = {
    sm: "px-2 py-0.5 text-[11px]",
    md: "px-2.5 py-1 text-xs",
  };

  const compactStatus = variant === "success" || variant === "error";

  return (
    <span
      className={`
        inline-flex items-center gap-1.5 font-medium rounded-full border whitespace-nowrap leading-none
        ${variants[variant]}
        ${sizes[size]}
        ${compactStatus ? "min-w-[3.5rem] justify-center" : ""}
      `}
    >
      {variant === "success" && <span className="w-1 h-1 rounded-full bg-emerald-400" />}
      {variant === "error" && <span className="w-1 h-1 rounded-full bg-red-400" />}
      {variant === "warning" && <span className="w-1 h-1 rounded-full bg-amber-400" />}
      {children}
    </span>
  );
}
