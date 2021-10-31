# -*- coding: utf-8 -*-
__author__ = "Paul Schifferer <dm@sweetrpg.com>"
"""
"""

import sentry_sdk
import os
from sweetrpg_main_web.application import constants
from sentry_sdk.integrations.redis import RedisIntegration
from sentry_sdk.integrations.flask import FlaskIntegration


sentry_sdk.init(dsn=os.environ[constants.SENTRY_DSN],
                environment=os.environ.get(constants.SENTRY_ENV) or 'Unknown',
                integrations=[
                    FlaskIntegration(), RedisIntegration()
                    ])
