import { FC, ComponentType, PropsWithChildren } from "react";
import clsx from "clsx";
import Link from "next/link";

import {
  Rocket,
  BookOpen,
  ArrowUpRight,
  Bookmark,
  UsersRound,
} from "lucide-react";

type Icon = "rocket" | "book" | "bookmarkSquare" | "userGroup";

const icons: Record<Icon, ComponentType<{ className?: string }>> = {
  rocket: Rocket,
  book: BookOpen,
  bookmarkSquare: Bookmark,
  userGroup: UsersRound,
};

const getIcon = (icon: Icon) => {
  return icons[icon];
};

interface CardProps {
  title: string;
  description?: string;
  icon?: Icon;
  href?: string;
  img?: string;
}

const Card: FC<CardProps> = ({ title, description, icon, href }: CardProps) => {
  const Icon = icon ? getIcon(icon) : null;

  const content = (
    <div className="group h-full w-full rounded-2xl p-1.5 shadow-xl ring-1 shadow-zinc-700/1 ring-zinc-200 transition-all duration-150 ease-in-out ring-inset hover:translate-y-[-2px] dark:shadow-black/5 dark:ring-zinc-800/70">
      <div
        className={clsx(
          "group relative h-full rounded-[11px] p-6 shadow-2xl ring-1 shadow-zinc-600/3 ring-zinc-300 ring-inset dark:shadow-black/5 dark:ring-zinc-800",
          href &&
            "cursor-pointer transition-all duration-100 ease-in-out group-hover:translate-y-[-3px] group-hover:shadow-zinc-600/10 dark:group-hover:shadow-black/40",
        )}
      >
        {href && (
          <ArrowUpRight className="absolute top-6 right-6 size-5 text-zinc-300 transition-all duration-100 ease-in-out group-hover:top-5 group-hover:right-5 group-hover:scale-105 group-hover:text-zinc-400 dark:text-zinc-600 dark:group-hover:text-zinc-400" />
        )}
        {Icon && (
          <Icon className="mb-3 size-[23px] text-zinc-500 dark:text-zinc-400" />
        )}
        <h3 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100">
          {title}
        </h3>
        <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
          {description}
        </p>
      </div>
    </div>
  );

  if (href) {
    return (
      <Link
        href={href}
        className="custom-tab-outline-offset-2 rounded-2xl transition-all duration-150 ease-in-out"
      >
        {content}
      </Link>
    );
  }

  return content;
};

interface CardGroupProps {
  columns: 2 | 3 | 4;
}

const CardGroup = ({
  columns,
  children,
}: PropsWithChildren<CardGroupProps>) => {
  return (
    <div
      className={clsx(
        "mt-4 grid gap-4",
        columns === 2 && "grid-cols-1 md:grid-cols-2",
        columns === 3 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
        columns === 4 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
      )}
    >
      {children}
    </div>
  );
};

export { Card, CardGroup };
