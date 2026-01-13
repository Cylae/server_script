import logging
from rich.logging import RichHandler
from rich.console import Console

console = Console()

def setup_logger(level="INFO"):
    logging.basicConfig(
        level=level,
        format="%(message)s",
        datefmt="[%X]",
        handlers=[RichHandler(console=console, rich_tracebacks=True)]
    )
    return logging.getLogger("cyl_manager")

logger = setup_logger()
