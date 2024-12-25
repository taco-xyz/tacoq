import sys
import importlib
import os
from pathlib import Path
from types import ModuleType
from typing import Dict
from watchfiles import awatch
from cli.logger import logger


class ModuleReloader:
    def __init__(self, import_path: str):
        self.import_path = import_path
        module_name = import_path.split(":")[0]
        self.base_module = sys.modules[module_name]
        # Change to include parent directories so other folders like "examples" are watched
        self.base_path = Path(self.base_module.__file__).parents[2].resolve()
        self.watched_modules: Dict[str, float] = {}
        self._update_dependency_tree()

    def _update_dependency_tree(self) -> None:
        """Update the full dependency tree of the application"""
        new_modules: Dict[str, float] = {}

        for name, mod in list(sys.modules.items()):
            if not self._is_valid_module(mod):
                continue

            try:
                mod_path = Path(mod.__file__).resolve()
                # Track if module is part of our project
                if self._is_project_module(mod_path):
                    new_modules[str(mod_path)] = os.path.getmtime(mod_path)
            except (AttributeError, TypeError):
                continue

        # Check for new or modified modules
        self.watched_modules = new_modules

    def _is_valid_module(self, module: ModuleType | None) -> bool:
        """Check if a module should be tracked"""
        return (
            module is not None
            and hasattr(module, "__file__")
            and module.__file__ is not None
            and module.__file__.endswith(".py")
        )

    def _is_project_module(self, mod_path: Path) -> bool:
        """Check if a module path belongs to our project"""
        try:
            return mod_path.is_relative_to(self.base_path)
        except ValueError:
            return False

    def _reload_module(self, module_path: str) -> None:
        """Reload a specific module and its dependencies"""
        try:
            rel_path = Path(module_path).resolve().relative_to(self.base_path)
            module_name = str(rel_path.with_suffix("")).replace(os.sep, ".")

            if module_name in sys.modules:
                logger.info(f"Reloading module: {module_name}")
                importlib.reload(sys.modules[module_name])
        except ValueError:
            pass  # Path not relative to base_path

    async def watch_and_reload(self) -> bool:
        """Watch for changes and reload modules. Returns True if changes detected."""
        async for changes in awatch(self.base_path):
            reload_needed = False

            for _, path in changes:
                if path.endswith(".py"):
                    self._reload_module(path)
                    reload_needed = True

            if reload_needed:
                # Update dependency tree to catch new imports
                self._update_dependency_tree()
                return True

        return False
