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
    # Host: dev.sweetrpg.com
    # X-Real-Ip: 10.32.0.7
    # Connection: close
    # User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:93.0) Gecko/20100101 Firefox/93.0
    # Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8
    # Accept-Encoding: gzip, deflate, br
    # Accept-Language: en
    # Cookie: session=63e6f102-0b63-4576-9d1a-c9fdba16ba13; sweetrpg-auth=SrSVWUQeCGl_2cFbOQkyNS40zXOF1u4OJp0KJE3y-5sjr6mUviiCJda2Evrd375UMxg_ohwft_QgRyMlG8f3kY66WhVyKaIaAkBoYD1ruxexScFDL8whGg1-aOVs4v0PRoEPMcrMylacJ0-hhT_TgXGvHqFSyf5HuQb61R046oL2WztqEDnv4LFnXGWmDwzsmAtklz6jxZCuK8P0vWuWpLMdUkBHdwth-R2L1pSxscrH0SLLDA_mQPVpb6cHOTchNdxGMtp7CB79T87i8uB3jV1nHywexzg-ghj9_eHsRa_jH_hoc5wWsmzAzPW8cNWB_bfVh4I7sbMtYhSek5bSU4aZ3QuZJGHRKr3GJquEOzfrLf2MFrJj5VhH0bIXHKB6YqR9HRiJwJyySokiKropncqxVuuAQofd0vvNXd6lnL2R7E1Oq5_YkEzmZqvzMbylgn2TMZ6cRmnLagqnZade8LDG0TTFcGo3khP4SuDpgkx2q27uz7CiD6c-WxadwE8uxVnQ6tfSa7vrX9zXUtm44N9gZLGndQg4W0tj3nO2C8UwvjiMmAyUbQXA7GKDmPu3RZhoQ22y1IbKc0zN0zQP3YKNCSaIoeQoT1182Gtzj4EunIt-9eRAmGxKK0MBOHCH2Wq0rqLq5Sq9NdDILE8NkAq6SQHTl4CE_HeGpueq6QRmvZ6kFQJT9OxOrwRL0z6Y89SljFQ-5HeAyYvP3wqmlSW1aJ4nSBDP14xv-I_YpTW7KXaBFf2qSoxplGxSNyUVGD_PHl95vxYb4MycsILbkis6ivwMnwh4q1r32dZkg8tmhsocHUPwo1OS7OzD-K6DPmOXIOwfIZAAh2Mg179GG6u3DKYdsmQ2Tpv4tXnKm9AaryoT34T1vYDt2b4sLD8xXh0w8AMLwO88Q4sp2N6b0W4XAkHWadtUqIdDPdMHJQuam5fM1EFtQw6KPmeG1yasdGKaHk8drHrCltIcPpkXGC1Cwjs_gzDBxm2p68CkFZ7hlMXXbezIsLrNjwosGqNqgPrrk18KNjP2kmR1pfomQxTJF4AifAAAB6s4VsrKZT02FDlmJJP6AiKTtJNovVcrgoWld1WjrmZp3y4Jj1cWOhV7ZcAe7T7sjPlweBC1eIn9vlO5vDDDbIkaNqYvTtv3NOuwLQN2azmAt1_Vu6cpzYitZrlps80NzPNilWU-zI7caeIAFQBqX40bwBZXrUX88dIgJxNHCPePOBU6uM5Taddqk9cpKBzcG7T0pwF4XDTwhoTOkACqAtC26kwSwxuB2jSG6ZdXotsnTi-GnFaKZX6c6g0f_A44v_0AWch0PNV2v0AdSri9nMMBRkQ2HtkmoptV3LzvBpRkzZ0Boxfh0MafR4ugAWquNdxLlBJWATuTogmpiYdLPRw=|1635722526|tm8Zo044MaWFCyyO97ZD5VXPZJLfM9y1cKiC4NMYQ6Y=; ajs_anonymous_id=%2205a30181-160f-4a76-b562-7610a47369ea%22
    # Dnt: 1
    # Sec-Fetch-Dest: document
    # Sec-Fetch-Mode: navigate
    # Sec-Fetch-Site: none
    # Sec-Fetch-User: ?1
    # Upgrade-Insecure-Requests: 1
    # X-Forwarded-Access-Token: C99_j-whvVVd3AgQ--S1RCQC7tpTNgd7
    # X-Forwarded-Email: paul@schifferers.net
    # X-Forwarded-For: 10.46.0.0, 10.32.0.3
    # X-Forwarded-Host: dev.sweetrpg.com
    # X-Forwarded-Port: 80
    # X-Forwarded-Proto: http
    # X-Forwarded-Server: traefik-m5tc5
    # X-Forwarded-User: github|419457
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
    session[constants.SESSION_ACCESS_TOKEN] = request.headers.get('X-Forwarded-Access-Token')
    session[constants.SESSION_EMAIL] = request.headers.get('X-Forwarded-Email')
    session[constants.SESSION_USER_ID] = request.headers.get('X-Forwarded-User')

    print(f"(updated) session: {session}")
    print(f"userinfo: {userinfo}")


@blueprint.before_request
def _store_user():
    email = session.get(constants.SESSION_EMAIL)
    print(f"email: {email}")
    user_id = session.get(constants.SESSION_USER_ID)
    print(f"user_id: {user_id}")
    if user_id and email:
        # TODO: store user
        pass


@blueprint.before_request
def _track():
    email = session.get(constants.SESSION_EMAIL)
    print(f"email: {email}")
    user_id = session.get(constants.SESSION_USER_ID)
    print(f"user_id: {user_id}")
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
    context = {
        'user_info': session.get(constants.SWEETRPG_AUTH_KEY)
    }

    print(f"context: {context}")
    return render_page("index.html", context)


from sweetrpg_web_core.blueprints import health
