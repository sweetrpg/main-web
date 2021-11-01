# -*- coding: utf-8 -*-
__author__ = "Paul Schifferer <dm@sweetrpg.com>"
"""
"""

from functools import wraps
from flask import redirect, session, render_template, request
from sweetrpg_main_web.application import constants
import jinja2
from flask import Blueprint, request, render_template, session, jsonify, current_app
from werkzeug.exceptions import HTTPException
import json
import os
from sweetrpg_main_web.application import constants
import analytics
import datetime

# def requires_auth(f):
#     @wraps(f)
#     def _check_auth(*args, **kwargs):
#         if constants.PROFILE_KEY not in session:
#             return redirect('/auth/login')
#         return f(*args, **kwargs)

#     return _check_auth


def tracked(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        print(f"args: {args}")
        print(f"kwargs: {kwargs}")
        print(f"session: {session}")
        print(f"request: {request}")

        userinfo = None
        if constants.SWEETRPG_AUTH_KEY in session:
            userinfo = session[constants.SWEETRPG_AUTH_KEY]
        elif constants.SWEETRPG_AUTH_KEY in request.cookies:
            userinfo = request.cookies[constants.SWEETRPG_AUTH_KEY]

        print(f"userinfo: {userinfo}")
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
        return render_page(f"errors/{code}.html")
    except jinja2.TemplateNotFound:
        return render_page("errors/error.html", context)


def render_page(page, context={}):
    show_cookie_message = True
    if request.cookies.get("cookies-accepted"):
        show_cookie_message = False
    context.update({
        "showCookieMessage": show_cookie_message,
    })

    userinfo = session.get(constants.PROFILE_KEY)
    if userinfo:
        context.update({
            "user_info": userinfo,
            "segment_write_key": os.environ.get(constants.SEGMENT_WRITE_KEY, "")
        })
    print(f"context: {context}")

    return render_template(page, **context)


class UserAuthorizationException(Exception):
    def __init__(self, reason: str):
        self.reason = reason


# def _check_user(role_name: str):
#     user_id = session.get(constants.CURRENT_USER_ID)
#     if user_id:
#         user = User.query.filter_by(id=user_id).first()
#         if user:
#             if has_role(user, role_name):
#                 return user

#             raise UserAuthorizationException('insufficient permissions')

#         raise UserAuthorizationException('user not found')

#     raise UserAuthorizationException('no user in session')


# def admin_required(f):
#     @wraps(f)
#     def _get_user(*args, **kwargs):
#         try:
#             user = _check_user(model_constants.ROLE_ADMIN)
#             return f(user, *args, **kwargs)
#         except UserAuthorizationException as e:
#             return jsonify({
#                 'error': "Unauthorized; " + e.reason
#             }), 401

#     return _get_user


# def user_required(f):
#     @wraps(f)
#     def _get_user(*args, **kwargs):
#         try:
#             user = _check_user(model_constants.ROLE_USER)
#             return f(user, *args, **kwargs)
#         except UserAuthorizationException as e:
#             return jsonify({
#                 'error': "Unauthorized; " + e.reason
#             }), 401

#     return _get_user


# def user_optional(f):
#     @wraps(f)
#     def _get_user(*args, **kwargs):
#         try:
#             user = _check_user(model_constants.ROLE_USER)
#             return f(user, *args, **kwargs)
#         except UserAuthorizationException as e:
#             return f(None, *args, **kwargs)

#     return _get_user


blueprint = Blueprint("web", __name__)


@blueprint.before_request
def _populate():
    print(f"session: {session}")
    print(f"headers: {request.headers}")
    print(f"cookies: {request.cookies}")
    print(f"args: {request.args}")

    userinfo = None
    if constants.PROFILE_KEY in session:
        userinfo = session[constants.PROFILE_KEY]
    elif constants.SWEETRPG_AUTH_KEY in request.cookies:
        userinfo = request.cookies[constants.SWEETRPG_AUTH_KEY]
        session[constants.PROFILE_KEY] = userinfo

    print(f"(updated) session: {session}")
    print(f"userinfo: {userinfo}")


@blueprint.before_request
def _track():
    user_info = session.get(constants.PROFILE_KEY)
    print(f"user_info: {user_info}")
    if user_info:
        analytics.identify('f4ca124298', {
            'name': 'Michael Bolton',
            'email': 'me@example.org',
            'created_at': datetime.datetime.now()
        })

        analytics.track('f4ca124298', request.url, {
            'plan': 'Enterprise'  # TODO
        })

@blueprint.errorhandler(Exception)
def error_handler(ex):
    current_app.logger.exception(f"Exception caught: {ex}")
    response = jsonify(message=str(ex))
    response.status_code = ex.code if isinstance(ex, HTTPException) else 500
    return response


@blueprint.route("/")
def main_page():
    context = {
        'user_info': session.get(constants.SWEETRPG_AUTH_KEY)
    }

    print(f"context: {context}")
    return render_page("index.html", context)


from sweetrpg_web_core.blueprints import health
