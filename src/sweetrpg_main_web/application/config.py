# -*- coding: utf-8 -*-
__author__ = "Paul Schifferer <dm@sweetrpg.com>"
"""
config.py
- settings for the flask application object
"""


import os
import redis
import random
import hashlib
from sweetrpg_main_web.application import constants


class BaseConfig(object):
    DEBUG = bool(os.environ.get(constants.DEBUG) or True)
    PORT = os.environ.get(constants.PORT) or 5000
    ASSETS_DEBUG = True
    LOG_LEVEL = os.environ.get(constants.LOG_LEVEL) or "INFO"
    # used for encryption and session management
    SECRET_KEY = os.environ.get('SECRET_KEY') or hashlib.sha256(f"{random.random()}".encode('utf-8')).hexdigest()
    CSRF_TOKEN = os.environ.get('CSRF_TOKEN') or hashlib.sha256(f"{random.random()}".encode('utf-8')).hexdigest()
    CACHE_REDIS_HOST = os.environ[constants.REDIS_HOST]
    CACHE_REDIS_PORT = int(os.environ.get(constants.REDIS_PORT) or 6379)
    CACHE_REDIS_DB = int(os.environ.get(constants.REDIS_DB) or 7)
    SESSION_TYPE = "redis"
    SESSION_REDIS = redis.from_url(f"redis://{os.environ[constants.REDIS_HOST]}:{int(os.environ.get(constants.REDIS_PORT) or 6379)}")
    SEGMENT_WRITE_KEY = os.environ.get(constants.SEGMENT_WRITE_KEY)
