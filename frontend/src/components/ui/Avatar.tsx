interface AvatarProps {
  name: string;
  src?: string;
  size?: "sm" | "md" | "lg" | "xl";
  className?: string;
}

export function Avatar({ name, src, size = "md", className = "" }: AvatarProps) {
  const sizes = {
    sm: "w-7 h-7 text-[10px]",
    md: "w-9 h-9 text-xs",
    lg: "w-12 h-12 text-sm",
    xl: "w-16 h-16 text-base",
  };

  const initial = name.charAt(0).toUpperCase();
  const gradient = getGradient(name);

  if (src) {
    return (
      <img
        src={src}
        alt={name}
        className={`${sizes[size]} rounded-full object-cover border border-gray-200 dark:border-white/[0.08] ${className}`}
      />
    );
  }

  return (
    <div
      className={`
        ${sizes[size]} ${gradient}
        rounded-full flex items-center justify-center
        font-medium text-white border border-white/10
        ${className}
      `}
    >
      {initial}
    </div>
  );
}

// Generate consistent subtle gradient based on name
function getGradient(name: string): string {
  const gradients = [
    "bg-gradient-to-br from-gray-700 to-gray-800",
    "bg-gradient-to-br from-gray-600 to-gray-700",
    "bg-gradient-to-br from-neutral-700 to-neutral-800",
    "bg-gradient-to-br from-slate-700 to-slate-800",
    "bg-gradient-to-br from-zinc-700 to-zinc-800",
  ];

  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }

  return gradients[Math.abs(hash) % gradients.length];
}
