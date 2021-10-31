# -*- coding: utf-8 -*-
__author__ = "Paul Schifferer <dm@sweetrpg.com>"
"""
user.py
- Utility functions for managing users.
"""


from flask import current_app, session
from sweetrpg_main_web.application import constants


def add_user_info(data: dict) -> dict:
    """Set user audit fields, such as `created_by` and `updated_by` with current user info.
    If the application is in debug mode, the system user ID is used.

    :param dict data: The data dictionary to update.
    :returns data: The data that was passed in, with user audit fields set.
    """
    if current_app.config['DEBUG']:
        data['updated_by'] = constants.SYSTEM_USER_ID
    elif session['user']:
        data['created_by'] = session['user']['id']

    if not data.get('created_by'):
        data['created_by'] = constants.SYSTEM_USER_ID

    return data

# def has_role(user: User, role_name: str):
#     for role_assoc in user.roles:
#         if role_assoc.enabled and role_assoc.roles.name == role_name:
#             return True

#     return False


# def create_or_add_user(userinfo: dict, role_name: str = model_constants.ROLE_USER):
#     subject = userinfo['sub']

#     # find the role
#     role = Role.query.filter_by(name=role_name).first()
#     if not role:
#         current_app.logger.error(f"Role {role_name} not found!")
#         return None, None

#     # find the user object
#     user = None

#     email = userinfo.get('email')
#     nickname = userinfo.get('nickname')
#     name = userinfo.get('name')

#     if email:
#         user = User.query.filter_by(email=email).first()
#         if not user:
#             user = User(email=email)
#             user.nickname = userinfo['nickname']
#             user.name = userinfo['name']
#             if userinfo.get('picture'):
#                 current_app.logger.info("Using provided picture URL for avatar.")
#                 user.avatar_url = userinfo['picture']
#             else:
#                 current_app.logger.info("Calculating hash for gravatar...")
#                 canonical_email = email.strip().lower()
#                 email_hash = hashlib.md5(canonical_email).hexdigest()
#                 current_app.logger.debug(f"email_hash: {email_hash}")
#                 user.avatar_url = f'https://www.gravatar.com/avatar/{email_hash}?s=30'
#     elif nickname:
#         pass # TODO
#     elif name:
#         pass # TODO
#     else:
#         # TODO
#         raise Exception('invalid user info')

#     if user:
#         db.session.add(user)
#         db.session.commit()

#         role_assoc = UserRole(user, role)
#         db.session.add(role_assoc)
#         db.session.commit()
#         print(user)

#         # find identity object
#         identity = Identity.query.filter_by(subject=subject).first()
#         if not identity:
#             source = subject.split('|', 2)[0]
#             identity = Identity(user=user, source=source, subject=subject)
#             # TODO

#             db.session.add(identity)
#             db.session.commit()
#         print(identity)

#     return user, identity
