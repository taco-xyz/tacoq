import { FC, ComponentType } from "react";
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
    <div className="w-full h-full ring-1 group rounded-2xl p-1.5  ring-zinc-200 dark:ring-zinc-800/70 shadow-xl shadow-zinc-700/1 dark:shadow-black/5 transition-all duration-150 ease-in-out ring-inset hover:translate-y-[-2px]">
      <div
        className={clsx(
          "rounded-[11px] relative group h-full ring-1 ring-inset ring-zinc-300 dark:ring-zinc-800 p-6 shadow-2xl shadow-zinc-600/3 dark:shadow-black/5",
          href &&
            "cursor-pointer transition-all duration-100 ease-in-out group-hover:translate-y-[-3px] group-hover:shadow-zinc-600/10 dark:group-hover:shadow-black/40"
        )}
      >
        {href && (
          <ArrowUpRight className="size-5 group-hover:text-zinc-400 dark:group-hover:text-zinc-400 transition-all duration-100 ease-in-out absolute group-hover:top-5 group-hover:right-5 top-6 right-6 text-zinc-300 dark:text-zinc-600" />
        )}
        {Icon && (
          <Icon className="size-6 mb-3 dark:text-zinc-400 text-zinc-500" />
        )}
        <h3 className="text-lg dark:text-zinc-100 text-zinc-900 font-semibold">
          {title}
        </h3>
        <p className="text-sm mt-1 dark:text-zinc-400 text-zinc-600">
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
  children: React.ReactNode;
}

const CardGroup: FC<CardGroupProps> = ({ columns, children }) => {
  return (
    <div
      className={clsx(
        "mt-4 grid gap-4",
        columns === 2 && "grid-cols-1 md:grid-cols-2",
        columns === 3 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
        columns === 4 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-4"
      )}
    >
      {children}
    </div>
  );
};

export { Card, CardGroup };
