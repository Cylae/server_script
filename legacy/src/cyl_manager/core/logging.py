import logging
from rich.logging import RichHandler
from rich.console import Console

console = Console()

def setup_logging(level: str = "INFO") -> logging.Logger:
    logging.basicConfig(
        level=level,
        format="%(message)s",
        datefmt="[%X]",
        handlers=[RichHandler(console=console, rich_tracebacks=True)]
    )
    return logging.getLogger("cyl_manager")

logger = setup_logging()

def print_header() -> None:
    console.print(r"""
   ______      __             __  ___
  / ____/_  __/ /  ____  ___ /  |/  /___ _____  ____ _____ ____  _____
 / /   / / / / /  / __ \/ _ \ /|_/ / __ `/ __ \/ __ `/ __ `/ _ \/ ___/
/ /___/ /_/ / /__/ /_/ /  __/ /  / / /_/ / / / / /_/ / /_/ /  __/ /
\____/\__, /____/\__,_/\___/_/  /_/\__,_/_/ /_/\__,_/\__, /\___/_/
     /____/                                         /____/
    """, style="bold cyan")
