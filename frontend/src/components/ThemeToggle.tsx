import { Moon, Sun, Monitor } from "lucide-react";
import { useThemeStore, type ThemeMode } from "#src/stores/theme";

const themeConfig: Record<ThemeMode, { icon: React.ReactNode; label: string }> = {
  light: { icon: <Sun className="w-4 h-4" />, label: "浅色" },
  dark: { icon: <Moon className="w-4 h-4" />, label: "深色" },
  auto: { icon: <Monitor className="w-4 h-4" />, label: "自动" },
};

export function ThemeToggle({ showLabel = false }: { showLabel?: boolean }) {
  const { mode, resolvedMode, setMode } = useThemeStore();

  return (
    <button
      type="button"
      onClick={() => setMode(mode === "light" ? "dark" : mode === "dark" ? "auto" : "light")}
      className="
        group inline-flex items-center gap-2
        px-2.5 py-2 rounded-lg
        text-gray-500
        hover:text-gray-300
        hover:bg-white/[0.04]
        transition-all duration-200
        focus:outline-none focus:ring-1 focus:ring-white/10
      "
      title={`当前: ${themeConfig[mode].label} (${resolvedMode === "dark" ? "深色" : "浅色"}模式)`}
    >
      <span className="transition-transform duration-200 group-hover:scale-110">
        {themeConfig[mode].icon}
      </span>
      {showLabel && <span className="text-sm font-medium">{themeConfig[mode].label}</span>}
    </button>
  );
}

export function ThemeSelector() {
  const { mode, setMode } = useThemeStore();

  const options: ThemeMode[] = ["light", "dark", "auto"];

  return (
    <div className="inline-flex p-1 bg-white/[0.04] rounded-lg border border-white/[0.08]">
      {options.map((option) => (
        <button
          key={option}
          type="button"
          onClick={() => setMode(option)}
          className={`
            flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium
            transition-all duration-200
            ${mode === option ? "bg-white/[0.08] text-white" : "text-gray-500 hover:text-gray-300"}
          `}
        >
          {themeConfig[option].icon}
          {themeConfig[option].label}
        </button>
      ))}
    </div>
  );
}
