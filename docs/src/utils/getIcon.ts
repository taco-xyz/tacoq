// Lucide Icons Imports
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

/**
 * Icons object containing all available Lucide icons
 */
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

/**
 * Retrieves an icon from the icons object based on the icon name
 * @param iconName - The name of the icon to retrieve
 * @returns The requested icon from the icons object
 */
export function getIcon(iconName: string) {
  return icons[iconName as keyof typeof icons];
}
