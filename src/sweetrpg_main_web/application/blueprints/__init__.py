# -*- coding: utf-8 -*-
__author__ = "Paul Schifferer <dm@sweetrpg.com>"
"""Main blueprint.
"""

from functools import wraps
import jinja2
from flask import Blueprint, request, render_template, session, jsonify, current_app
from werkzeug.exceptions import HTTPException
import os
from sweetrpg_main_web.application import constants
from sweetrpg_web_core import constants as core_constants
import analytics
import datetime
from sweetrpg_web_core.helpers.context import get_context, populate_session


def tracked(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        current_app.logger.debug(f"args: {args}")
        current_app.logger.debug(f"kwargs: {kwargs}")
        current_app.logger.debug(f"session: {session}")
        current_app.logger.debug(f"request: {request}")

        userinfo = None
        if constants.SWEETRPG_AUTH_KEY in session:
            userinfo = session[constants.SWEETRPG_AUTH_KEY]
        elif constants.SWEETRPG_AUTH_KEY in request.cookies:
            userinfo = request.cookies[constants.SWEETRPG_AUTH_KEY]

        current_app.logger.debug(f"userinfo: {userinfo}")
        if userinfo:
            kwargs.update({
                'userinfo': userinfo,
            })

            analytics.identify('f4ca124298', {
                'name': 'Michael Bolton',
                'email': userinfo,
                'created_at': datetime.datetime.now()
            })

            analytics.track('f4ca124298', 'Main Page', {
                'plan': 'Enterprise'
            })

        return f(*args, **kwargs)

    return decorated


def error_page(message, code):
    context = {
        "code": code,
        "message": message,
    }
    try:
        return render_page(f"common/errors/{code}.html")
    except jinja2.TemplateNotFound:
        return render_page("common/errors/error.html", context)


def render_page(page:str, context:dict={}):
    """Call `render_template` for the specified page, and merge the
    provided context into an initialized, common context.

    :param str page:
    :param dict context:
    :returns:
    """
    show_cookie_message = True
    if request.cookies.get("cookies-accepted"):
        show_cookie_message = False
    context.update({
       "showCookieMessage": show_cookie_message,
    })

    userinfo = session.get(core_constants.PROFILE_KEY)
    if userinfo:
        context.update({
            "user_info": userinfo,
            "segment_write_key": os.environ.get(constants.SEGMENT_WRITE_KEY, "")
        })

    current_app.logger.debug(f"context: {context}")
    return render_template(page, **context)


class UserAuthorizationException(Exception):
    def __init__(self, reason: str):
        self.reason = reason


blueprint = Blueprint("web", __name__)


@blueprint.before_request
def _populate_session():
    populate_session()

    userinfo = None
    if core_constants.PROFILE_KEY in session:
        current_app.logger.info("Setting user info from current session.")
        userinfo = session[core_constants.PROFILE_KEY]

    current_app.logger.debug(f"(updated) session: {session}")
    current_app.logger.debug(f"userinfo: {userinfo}")


@blueprint.before_request
def _store_user():
    email = session.get(core_constants.SESSION_EMAIL)
    current_app.logger.debug(f"email: {email}")
    user_id = session.get(core_constants.SESSION_USER_ID)
    current_app.logger.debug(f"user_id: {user_id}")
    if user_id and email:
        # TODO: store user
        pass


@blueprint.before_request
def _track():
    email = session.get(core_constants.SESSION_EMAIL)
    current_app.logger.debug(f"email: {email}")
    user_id = session.get(core_constants.SESSION_USER_ID)
    current_app.logger.debug(f"user_id: {user_id}")
    if user_id and email:
        analytics.identify(user_id, {
            'email': email,
            'created_at': datetime.datetime.now()
        })

        analytics.track(user_id, request.full_path, {
            'user_agent': request.headers.get('User-Agent')
        })


@blueprint.errorhandler(Exception)
def error_handler(ex):
    current_app.logger.exception(f"Exception caught: {ex}")
    response = jsonify(message=str(ex))
    response.status_code = ex.code if isinstance(ex, HTTPException) else 500
    return response


@blueprint.route("/")
def main_page():
    context = get_context()
    context.update({
        # 'user_info': session.get(constants.SWEETRPG_SESSION_USER_INFO)
        'appname': "Main",
    })

    current_app.logger.debug(f"context: {context}")
    return render_page("main/index.html", context=context)


from sweetrpg_web_core.blueprints import health
