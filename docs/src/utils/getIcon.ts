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
};

export function getIcon(iconName: string) {
  return icons[iconName as keyof typeof icons];
}
