class ModuleWrapper:
    def __init__(self, module_name):
        self.module = __import__(module_name)

    def load(self):
        if hasattr(self.module, 'load'):
            return self.module.load()
        return "Module loaded"

    def unload(self):
        if hasattr(self.module, 'unload'):
            return self.module.unload()
        return "Module unloaded"

    def translate(self, text, source_lang, target_lang):
        if hasattr(self.module, 'translate'):
            return self.module.translate(text, source_lang, target_lang)
        raise NotImplementedError("Translate method not implemented for this module")