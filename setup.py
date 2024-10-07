from setuptools import setup

# Metadata goes in setup.cfg. These are here for GitHub's dependency graph.
setup(
    name="sweetrpg-main-web",
    install_requires=[
        "analytics-python~=1.4",
        "blinker~=1.0",
        "dnspython~=2.0",
        "Flask-Caching~=2.0",
        "Flask-CORS~=5.0",
        "Flask-DotEnv~=0.1",
        "Flask-Session~=0.4",
        "Flask~=3.0",
        "hiredis~=3.0",
        "kanka~=0.1",
        "python-dateutil~=2.0",
        "python-dotenv~=1.0",
        "python-editor~=1.0",
        "PyYAML~=6.0",
        "hiredis~=3.0",
        "requests~=2.0",
        "sentry-sdk[flask]==1.23",
        "SQLAlchemy~=1.4",
        "sweetrpg-web-core",
        "urllib3~=2.0",
    ],
    extras_require={},
)
