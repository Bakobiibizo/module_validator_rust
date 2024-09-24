import os
import json
import jsonpickle
import base64
from pathlib import Path
from typing import Dict, List, Union
from dotenv import load_dotenv
from cryptography.hazmat.primitives import serialization
from loguru import logger
from module_registrar.utilities.encryption import (
    derive_rsa_keypair_with_password,
   # encrypt_with_rsa_file,
   # decrypt_with_rsa_file,
    encode_ss58_address,
    PUBLIC_KEY,
    PRIVATE_KEY
)

load_dotenv()

class ModuleRegistrar:
    def __init__(self, module_name, target_modules_path, module_storage_path):
        self.setup_file_paths(module_name, target_modules_path, module_storage_path)
        self.load_registry()
        self.ignore_list = (".venv", "data/", ".", "__py", "node_modules")
        self.init_key(self.key_path)

    def setup_file_paths(self, module_name, target_modules_path, module_storage_path):
        self.ensure_directory_exists(module_storage_path)
        self.storage_path = Path(module_storage_path)
        self.target_modules_path = target_modules_path
        self.target_module_path = Path(self.target_modules_path) / module_name
        self.target_module_file_path = self.target_module_path / f"{module_name}_module.py"
        self.module_storage_path = self.storage_path / module_name
        self.module_setup_path = self.storage_path / module_name /  f"setup_{module_name}.py"
        self.registry_path = self.storage_path / "registry.json"
        self.key_path = Path(os.getenv('KEY_FOLDER', 'keys'))

    @staticmethod
    def ensure_directory_exists(path):
        Path(path).mkdir(parents=True, exist_ok=True)

    def load_registry(self):
        if self.registry_path.exists():
            self.registry = json.loads(self.registry_path.read_text(encoding="utf-8"))
            self.registry["public_key"] = PUBLIC_KEY.read_bytes().decode("utf-8")
        else:
            self.registry = {}

    def init_key(self, key_path=Path("data/private_key.pem")):
        if not key_path.exists():
            derive_rsa_keypair_with_password()

    def register(self, name):
        self.registry[name] = self.module_setup_path.read_text() # self.encrypt_with_rsa_file(self.module_setup_path.read_text(encoding="utf-8"))
        self.save_registry()

    def save_registry(self):
        self.ensure_directory_exists(self.registry_path.parent)
        for key, value in self.registry.items():
            if isinstance(value, bytes):
                value = base64.b64encode(value).decode('utf-8')
            self.registry[key] = json.dumps(value)
        self.registry_path.write_text(json.dumps(self.registry, indent=4), encoding="utf-8")

    def get(self, name):
        encoded_value = self.registry.get(name)
        return encoded_value

    def add_module(self, name, module_path):
        logger.debug(module_path)
        if not module_path:
            os.makedirs(f"modules/{name}", exist_ok=True)
        self.generate_script(name, module_path)
        self.register(name)

    def update_module(self, name, module_path):
        self.add_module(name, module_path)

    def remove_module(self, name):
        self.registry.pop(name, None)
        self.save_registry()
        os.rmdir(f"modules/{name}")

    def list_modules(self):
        return list(self.registry.keys())

    def walk_and_encode(self, dirpath):
        file_data = []
        logger.debug(f"Starting walk_and_encode for {dirpath}")
        logger.debug(f"Ignore list: {self.ignore_list}")
        for root, dirs, files in os.walk(dirpath):
            logger.debug(f"Examining directory: {root}")
            logger.debug(f"Subdirectories before filtering: {dirs}")
            dirs[:] = [d for d in dirs if not any(d.startswith(ignore) for ignore in self.ignore_list)]
            logger.debug(f"Subdirectories after filtering: {dirs}")
            logger.debug(f"Files: {files}")

            for file in files:
                if file.endswith(('.sh', '.py')) and not any(file.startswith(ignore) for ignore in self.ignore_list):
                    file_path = Path(root) / file
                    try:
                        relative_path = file_path.relative_to(dirpath)
                        content = file_path.read_bytes()
                        encoded_content = base64.b64encode(content).decode('utf-8')
                        file_data.append((str(relative_path), encoded_content))
                        logger.debug(f"Created: {relative_path}")
                    except ValueError as e:
                        logger.debug(f"Error processing {file_path}: {e}")

        logger.debug(f"Finished walk_and_encode. File data length: {len(file_data)}")
        return file_data

    def generate_script(self, name, folder_path):
        logger.info(f"Generating script for {name}")
        logger.info(f"Folder path: {folder_path}")
        file_data = self.walk_and_encode(folder_path)
        script_content = self.create_script_content(name, file_data)
        encoded_content = base64.b64encode(script_content.encode('utf-8')).decode('utf-8') 
        self.ensure_directory_exists(self.module_setup_path.parent)
        self.module_setup_path.write_text(encoded_content, encoding='utf-8')

    @staticmethod
    def create_script_content(name, file_data):
        lines = [
            "import os\n",
            "import base64\n",
            f"folder_path = f'modules/{name}'\n\n",
            "file_data = [\n",
            *[f"    ('{path}', '{content}'),\n" for path, content in file_data],
            "]\n\n",
            "for relative_path, encoded_content in file_data:\n",
            f"    full_path = os.path.join(folder_path, relative_path)\n",
            "    os.makedirs(os.path.dirname(full_path), exist_ok=True)\n",
            "    with open(full_path, 'wb') as f:\n",
            "        f.write(base64.b64decode(encoded_content))\n",
            "    print(f'Created: {full_path}')\n"
        ]
        return "".join(lines)

    #def encrypt_with_rsa_file(self, data):
    #    return encrypt_with_rsa_file(data=data)
    #
#
    #def decrypt_with_rsa_file(self, data):
    #    return decrypt_with_rsa_file(data=data)

    @staticmethod
    def save_key(key_path, key_data):
        key_path.write_bytes(key_data)

    def load_key(self):
        with open(PRIVATE_KEY, "rb") as f:
            return f.read()


def cli():
    module_name = input("Module_name: ")
    registrar = ModuleRegistrar(module_name, f"module_registrar/modules/{module_name}", "modules")

    options = {
        "1": ("Add Module", lambda: registrar.add_module(module_name, f"module_registrar/modules/{module_name}")),
        "2": ("Update Module", lambda: registrar.update_module(module_name, f"module_registrar/modules/{module_name}")),
        "3": ("Remove Module", lambda: registrar.remove_module(module_name)),
        "4": ("List Modules", lambda: print("Modules: ", registrar.list_modules())),
        "5": ("Exit", lambda: exit(0))
    }

    while True:
        print("\nModule Registry:", registrar.registry)
        print("\nOptions:")
        for key, (description, _) in options.items():
            print(f"{key}. {description}")

        choice = input("Enter your choice: ")
        if choice in options:
            options[choice][1]()
        else:
            print("Invalid choice. Please try again.")

if __name__ == "__main__":
    cli()