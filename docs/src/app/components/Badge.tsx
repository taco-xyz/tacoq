interface BadgeProps {
  title: string;
  Icon?: React.ComponentType<{ className?: string }>;
}

export default function Badge({ title, Icon }: BadgeProps) {
  return (
    <div className="flex flex-row items-center w-fit justify-center gap-x-1.5 text-sm font-medium rounded-full text-zinc-600 dark:text-zinc-400 transition-colors duration-150 ease-in-out">
      {Icon && <Icon className="size-3.5" />}
      {title}
    </div>
  );
}
