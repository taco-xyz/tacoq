import {
  AcademicCapIcon,
  AdjustmentsVerticalIcon,
  ArrowDownTrayIcon,
  ArrowUpIcon,
  BoltIcon,
  BookOpenIcon,
  BuildingLibraryIcon,
  ChartBarIcon,
  ChevronDoubleRightIcon,
  CpuChipIcon,
  PencilSquareIcon,
  PresentationChartLineIcon,
  RocketLaunchIcon,
} from "@heroicons/react/24/outline";

const icons = {
  AcademicCapIcon,
  AdjustmentsVerticalIcon,
  ArrowDownTrayIcon,
  ArrowUpIcon,
  BoltIcon,
  BookOpenIcon,
  BuildingLibraryIcon,
  ChartBarIcon,
  ChevronDoubleRightIcon,
  CpuChipIcon,
  PencilSquareIcon,
  PresentationChartLineIcon,
  RocketLaunchIcon,
};

export function getIcon(iconName: string) {
  return icons[iconName as keyof typeof icons];
}
