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
  Layers2,
  Map,
  Coffee,
  Package,
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
  Layers2,
  Map,
  Coffee,
  Package,
};

export function getIcon(iconName: string) {
  return icons[iconName as keyof typeof icons];
}
