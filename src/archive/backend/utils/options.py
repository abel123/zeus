from pydantic import BaseModel
from backend.utils.magic import SingletonABCMeta


class Options(metaclass=SingletonABCMeta):
    class Config(BaseModel):
        enable_overnight: bool = False

    def __init__(self) -> None:
        self._config = Options.Config()

    def config(self) -> Config:
        return self._config
