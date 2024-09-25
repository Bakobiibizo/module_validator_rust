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

    def process(self, *args, **kwargs):
        if hasattr(self.module, 'process'):
            return self.module.process(*args, **kwargs)
        return "Module process failed"
