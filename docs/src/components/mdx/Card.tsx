import { FC, ComponentType } from "react";
import clsx from "clsx";
import Link from "next/link";
import {
  RocketLaunchIcon,
  BookOpenIcon,
  ArrowUpRightIcon,
  BookmarkSquareIcon,
  UserGroupIcon,
} from "@heroicons/react/24/outline";

type Icon = "rocket" | "book" | "bookmarkSquare" | "userGroup";

const icons: Record<Icon, ComponentType<{ className?: string }>> = {
  rocket: RocketLaunchIcon,
  book: BookOpenIcon,
  bookmarkSquare: BookmarkSquareIcon,
  userGroup: UserGroupIcon,
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
    <div
      className={clsx(
        "rounded-lg relative group h-full ring-1 ring-inset dark:ring-zinc-800 ring-zinc-300 p-6 dark:shadow-none shadow-2xl shadow-zinc-600/3",
        href &&
          "hover:ring-zinc-400 dark:hover:ring-zinc-400 cursor-pointer hover:ring-2 transition-all duration-100 ease-in-out"
      )}
    >
      {href && (
        <ArrowUpRightIcon className="size-4 group-hover:text-zinc-500 dark:group-hover:text-zinc-400 transition-all duration-100 ease-in-out absolute group-hover:top-5 group-hover:right-5 top-6 right-6 text-zinc-400 dark:text-zinc-600" />
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
  );

  if (href) {
    return <Link href={href}>{content}</Link>;
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
