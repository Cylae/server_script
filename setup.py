from setuptools import setup, find_packages

setup(
    name="cyl_manager",
    version="2.0.0",
    packages=find_packages(),
    install_requires=[
        "distro",
        "requests",
        "python-dotenv",
        "psutil",
        "docker",
        "jinja2",
        "rich",
        "typer",
        "pydantic-settings"
    ],
    entry_points={
        "console_scripts": [
            "cyl-manager=cyl_manager.cli:app",
        ],
    },
)
