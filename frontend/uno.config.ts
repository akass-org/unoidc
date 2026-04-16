import { defineConfig, presetUno, presetAttributify } from "unocss";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  presets: [presetUno(), presetAttributify()],
  theme: {
    colors: {
      // Primary: Refined Indigo
      primary: {
        50: "#eef2ff",
        100: "#e0e7ff",
        200: "#c7d2fe",
        300: "#a5b4fc",
        400: "#818cf8",
        500: "#6366f1",
        600: "#4f46e5",
        700: "#4338ca",
        800: "#3730a3",
        900: "#312e81",
        950: "#1e1b4b",
      },
      // Neutral: Warm Slate
      slate: {
        50: "#f8fafc",
        100: "#f1f5f9",
        200: "#e2e8f0",
        300: "#cbd5e1",
        400: "#94a3b8",
        500: "#64748b",
        600: "#475569",
        700: "#334155",
        800: "#1e293b",
        900: "#0f172a",
        950: "#020617",
      },
      // Accent: Soft Emerald for success states
      success: {
        50: "#ecfdf5",
        100: "#d1fae5",
        500: "#10b981",
        600: "#059669",
      },
      // Error: Warm Rose
      error: {
        50: "#fff1f2",
        100: "#ffe4e6",
        500: "#f43f5e",
        600: "#e11d48",
      },
    },
    fontFamily: {
      sans: 'Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      mono: "JetBrains Mono, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    },
    boxShadow: {
      soft: "0 1px 3px rgba(0, 0, 0, 0.05), 0 1px 2px rgba(0, 0, 0, 0.1)",
      card: "0 1px 3px rgba(0, 0, 0, 0.1), 0 4px 6px -1px rgba(0, 0, 0, 0.05), 0 2px 4px -2px rgba(0, 0, 0, 0.05)",
      elevated:
        "0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.05)",
      glow: "0 0 20px rgba(99, 102, 241, 0.15)",
    },
    borderRadius: {
      sm: "0.375rem",
      DEFAULT: "0.5rem",
      md: "0.625rem",
      lg: "0.75rem",
      xl: "1rem",
      "2xl": "1.25rem",
    },
    transitionDuration: {
      fast: "100ms",
      DEFAULT: "150ms",
      slow: "250ms",
    },
  },
  shortcuts: {
    // Form elements
    "input-field": `
      w-full px-4 py-2.5 bg-white dark:bg-slate-900
      border border-slate-200 dark:border-slate-700
      rounded-lg text-sm text-slate-900 dark:text-slate-100
      placeholder:text-slate-400 dark:placeholder:text-slate-500
      transition-all duration-150 ease-out
      focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500
      hover:border-slate-300 dark:hover:border-slate-600
    `,
    "btn-primary": `
      inline-flex items-center justify-center
      px-4 py-2.5 bg-primary-600 hover:bg-primary-700
      text-white text-sm font-medium
      rounded-lg transition-all duration-150 ease-out
      focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:ring-offset-2
      disabled:opacity-50 disabled:cursor-not-allowed
      active:scale-[0.98]
    `,
    "btn-secondary": `
      inline-flex items-center justify-center
      px-4 py-2.5 bg-white dark:bg-slate-800
      border border-slate-200 dark:border-slate-700
      text-slate-700 dark:text-slate-200 text-sm font-medium
      rounded-lg transition-all duration-150 ease-out
      hover:bg-slate-50 dark:hover:bg-slate-700
      focus:outline-none focus:ring-2 focus:ring-slate-500/20
      active:scale-[0.98]
    `,
    // Card
    card: `
      bg-white dark:bg-slate-900
      rounded-xl border border-slate-200 dark:border-slate-800
      shadow-card
    `,
  },
  rules: [["text-balance", { "text-wrap": "balance" }]],
});
