import {
  GraduationCap,
  SlidersHorizontal,
  ArrowDownToLine,
  ArrowUp,
  Zap,
  BookOpen,
  Building2,
  ChartBar,
  ChevronRight,
  Cpu,
  PenSquare,
  ChartNoAxesCombined,
  Rocket,
  Cog,
} from "lucide-react";

const icons = {
  GraduationCap,
  SlidersHorizontal,
  ArrowDownToLine,
  ArrowUp,
  Zap,
  BookOpen,
  Building2,
  ChartBar,
  ChevronRight,
  Cpu,
  PenSquare,
  ChartNoAxesCombined,
  Rocket,
  Cog,
};

export function getIcon(iconName: string) {
  return icons[iconName as keyof typeof icons];
}
