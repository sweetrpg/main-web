from setuptools import setup

# Metadata goes in setup.cfg. These are here for GitHub's dependency graph.
setup(
    name="sweetrpg-main-web",
    install_requires=[
        "Authlib",
        "blinker",
        "dnspython<2.0.0",
        "Flask-Caching",
        "Flask-CORS",
        "Flask-DotEnv",
        "Flask-Session",
        "Flask==2.0.2",
        "gunicorn",
        "kanka",
        "python-dateutil",
        "python-dotenv",
        "python-editor",
        "PyYAML",
        "redis",
        "requests",
        "sentry-sdk",
        "analytics-python<2.0",
        "sweetrpg-web-core",
    ],
    extras_require={},
)
