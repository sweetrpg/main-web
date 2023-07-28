from setuptools import setup

# Metadata goes in setup.cfg. These are here for GitHub's dependency graph.
setup(
    name="sweetrpg-main-web",
    install_requires=[
        "analytics-python~=1.4",
        "blinker~=1.5",
        "dnspython~=2.2",
        "Flask-Caching~=1.11",
        "Flask-CORS~=3.0",
        "Flask-DotEnv~=0.1",
        "Flask-Session~=0.4",
        "Flask~=2.0",
        "hiredis~=2.0",
        "kanka~=0.1",
        "python-dateutil~=2.8",
        "python-dotenv~=0.21",
        "python-editor~=1.0",
        "PyYAML~=6.0",
        "redis~=4.3",
        "requests~=2.31",
        "sentry-sdk[flask]==1.23",
        "SQLAlchemy~=1.4",
        "sweetrpg-web-core",
        "urllib3~=1.26",
    ],
    extras_require={},
)
