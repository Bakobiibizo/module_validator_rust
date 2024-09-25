import importlib
import subprocess
from pathlib import Path
import sys
import os

CWD = Path.cwd()

class ModuleWrapper:
    def __init__(self, module_name):
        self.module = importlib.import_module(f"modules.{module_name}")
        self.module_name = module_name
        self.root_dir = Path(__file__).parent.parent.resolve()  # Get the absolute path to the project root
        self.venv_python = str(self.root_dir / ".venv" / "bin" / "python")
        if sys.platform == "win32":
            self.venv_python = str(self.root_dir / ".venv" / "Scripts" / "python.exe")

    def load(self):
        if hasattr(self.module, 'load'):
            return self.module.load()
        return "Module loaded"

    def unload(self):
        if hasattr(self.module, 'unload'):
            return self.module.unload()
        return "Module unloaded"

    def install(self, *args, **kwargs):
        if (self.root_dir / "modules" / self.module_name / f"setup_{self.module_name}.sh").exists():
            setup_script = self.root_dir / "modules" / self.module_name / f"setup_{self.module_name}.sh"
            command = ["bash", str(setup_script)]
            try:
                subprocess.run(command, check=True, cwd=str(self.root_dir))
                return "Module setup successfully"
            except subprocess.CalledProcessError as e:
                return f"Failed to setup module: {e}"
        elif (self.root_dir / "modules" / self.module_name / f"setup_{self.module_name}.py").exists():
            setup_script = self.root_dir / "modules" / self.module_name / f"setup_{self.module_name}.py"
            command = [self.venv_python, str(setup_script)]
            try:
                subprocess.run(command, check=True, cwd=str(self.root_dir))
                return "Module setup successfully"
            except subprocess.CalledProcessError as e:
                return f"Failed to setup module: {e}"
        else:
            return "Module setup failed"

    def process(self, *args, **kwargs):
        if hasattr(self.module, "process"):
            return self.module.process(*args, **kwargs)
        return "Module process failed"
