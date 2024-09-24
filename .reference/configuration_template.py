import json
import argparse
from pydantic import BaseModel, ConfigDict, Field
from typing import Dict, Any, Union, TypeVar, List, Optional
import bittensor as bt
from bittensor.metagraph import TorchMetaGraph, NonTorchMetagraph

T = TypeVar("T")
bt.metagraph

CONFIGURATION_TEMPLATE = """from pydantic import BaseModel, Field
from typing import Any, Union, Dict, List, Optional, ClassVar, TypeVar
from pydantic import ConfigDict
from dotenv import load_dotenv
from module_validator.config.base_configuration import GenericConfig, T
import bittensor as bt
import argparse
import os

load_dotenv()

<<sub_class_generation>>

class Config(GenericConfig):
    model_config: ClassVar[ConfigDict] = ConfigDict({
            "aribtrary_types_allowed": True
    })
    config: Optional[bt.config] = Field(default_factory=bt.config, type=None)
    axon: Optional[bt.axon] = Field(default_factory=bt.axon, type=None)
    wallet: Optional[bt.wallet] = Field(default_factory=bt.wallet, type=None)
    metagraph: Optional[T] = Field(default_factory=TypeVar, type=None)
    subtensor: Optional[bt.subtensor] = Field(default_factory=bt.subtensor, type=None)
    dendrite: Optional[bt.dendrite] = Field(default_factory=bt.dendrite, type=None)
    hotkeypair: Optional[bt.Keypair] = Field(default_factory=bt.Keypair, type=None)
<<attribute_generation>>
    
    def __init__(self, data: Union[BaseModel, Dict[str, Any]]):
        if isinstance(data, BaseModel):
            data = data.model_dump()
        super().__init__()

    def get(self, key: str, default: T = None) -> T:
        return self._get(key, default)
    
    def set(self, key: str, value: T) -> None:
        self._set(key, value)
        
    def merge(self, new_config: Dict[str, T]) -> Dict[str, Any]:
        self.config = self._merge(new_config, self.config)
        return self.config

    def load_config(self, parser: argparse.ArgumentParser, args: argparse.Namespace) -> 'Config':
        return self._load_config(parser, args)
    
    def parse_args(self, args: argparse.Namespace):
        self._parse_args(args)
    
    def add_args(self, parser: argparse.ArgumentParser) -> argparse.ArgumentParser:
        return self._add_args(parser)
    
    def get_env(self) -> List[str]:
        lines = [
<<environment_generation>>
        ]
        return self._add_env(self.config)

    def add_args(self, parser: argparse.ArgumentParser) -> argparse.ArgumentParser:
        parser.add_argument('--config', type=str, default=None, help='path to config file', required=False)
<<argument_generation>>
        return parser"""

class GenericConfig(BaseModel):
    file: Optional[str] = Field(default_factory=str, type=str)
    hotkey: Optional[str] = Field(default_factory=str, type=str)
    message: Optional[Dict[str, Any]] = Field(default_factory=dict, type=dict)
    miner: Optional[bt.axon] = Field(default_factory = bt.axon, type=bt.axon)
    mock: Optional[bool] = Field(defaul=True, type=bool)
    my_uid: Optional[int] = Field(default_factory=int, type=int)
    netuid: Optional[int] = Field(default_factory=int, type=int)
    network: Optional[str] = Field(default_factory=str, type=str)
    neuron: Optional[Dict[str, Any]] = Field(default_factory=dict, type=dict)
    wallet_name = Optional[str] = Field(default_factory=str, type=str)
    wandb: Optional[Dict[str, Any]] = Field(default_factory=dict, type=dict)
    hotkeypair: Optional[bt.Keypair] = Field(default_factory=bt.Keypair, type=None)
    config: Optional[Dict[str, Any]] = Field(default_factory=dict, type=None)
    axon: Optional[bt.axon] = Field(default_factory=bt.axon, type=None)
    wallet: Optional[bt.wallet] = Field(default_factory=bt.wallet, type=None)
    metagraph: Optional[Union[type[TorchMetaGraph], type[NonTorchMetagraph]]] = Field(default_factory=Union[TorchMetaGraph, NonTorchMetagraph], type=None)
    subtensor: Optional[bt.subtensor] = Field(default_factory=bt.subtensor, type=None)
    dendrite: Optional[bt.dendrite] = Field(default_factory=bt.dendrite, type=None)
    hotkeypair: Optional[bt.Keypair] = Field(default_factory=bt.Keypair, type=None)
    model_config: ConfigDict = ConfigDict({
        "arbitrary_types_allowed": True
    })
    __pydantic_fields_set__ = set([
        "file",
        "hotkey",
        "message",
        "miner",
        "mock",
        "my_uid",
        "netuid",
        "network",
        "neuron",
        "wallet_name",
        "wandb",
        "hotkeypair",
        "config",
        "axon",
        "wallet",
        "metagraph",
        "subtensor",
        "dendrite",
        "hotkeypair"
    ])

    def __init__(self, data: Dict[str, Any]=None):
        super(BaseModel).__init__(data)
        self.config = {}
        config_data = self.config or data
        self.config = self._merge(config_data, self.config) if self.config else config_data
        

    @classmethod
    def _get(cls, key: str, default: Any = None) -> Any:
        keys = key.split(".")
        value = cls.config
        for k in keys:
            if isinstance(value, dict):
                value = value.get(k)
            else:
                return default
            if value is None:
                return default
        return value

    @classmethod
    def _set(cls, key: str, value: Any):
        keys = key.split(".")
        d = cls.config
        for k in keys[:-1]:
            if k not in d or not isinstance(d[k], dict):
                d[k] = {}
            d = d[k]
        d[keys[-1]] = value

    @classmethod
    def _merge(
        cls, new_config: Dict[str, Any], old_config: Dict[str, Any]
    ) -> Dict[str, Any]:
        merged_config = old_config.copy()
        for key, value in new_config.items():
            if (
                isinstance(value, dict)
                and key in merged_config
                and isinstance(merged_config[key], dict)
            ):
                merged_config[key] = cls._merge(value, merged_config[key])
            else:
                merged_config[key] = value
        return merged_config

    @classmethod
    def _load_config(
        cls, parser: argparse.ArgumentParser, args: argparse.Namespace
    ) -> "GenericConfig":
        config = cls(config={})
        args = parser.parse_args(args) if args else parser.parse_args()
        config._parse_args(args)
        return config

    @classmethod
    def _parse_args(cls, args: argparse.Namespace):
        for arg, value in vars(args).items():
            if value is not None:
                cls._set(arg, value)

    @classmethod
    def _prompt_for_value(cls, key: str, default: Any = None) -> Any:
        if key in cls._config:
            value = input(f"Enter value for {key}[{default}]: ")
            cls.__set(key, value)
            return value

if __name__ == "__main__":

    config = GenericConfig()
