import requests
import base64
import json

def install_module(module_name, url):
    response = requests.get(url)
    if response.status_code == 200:
        module_data = response.json()
        script = base64.b64decode(json.loads(module_data)).decode('utf-8')
        with open(f'modules/{module_name}.py', 'w') as file:
            file.write(script)
        print(f"Module {module_name} installed successfully.")
    else:
        print(f"Failed to install module {module_name}. Status code: {response.status_code}")

if __name__ == "__main__":
        install_module("translation", "https://registrar-agentartificial.ngrok.dev/modules/translation")