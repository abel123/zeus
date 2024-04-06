from sanic import Sanic

from backend.api.option_price import OptionTracker


class Registry:
    def __init__(self, app: Sanic) -> None:
        app.add_route(
            OptionTracker().option_price, "/symbol/option_price", "symbol_option_price"
        )
