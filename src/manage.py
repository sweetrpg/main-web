__author__ = "Paul Schifferer <dm@sweetrpg.com>"
"""
manage.py
- provides a command line utility for interacting with the
  application to perform interactive debugging and setup
"""

from flask_script import Manager
from flask_migrate import Migrate, MigrateCommand

from application.main import create_app
from application.db import db
from application.models.common.game_system import GameSystem, GameSystemFacetDatum, GameSystemImageDatum
from application.models.initiative.condition import Condition, ConditionHealthAdjustment
from application.models.initiative.encounter import Encounter, EncounterParticipant, EncounterParticipantGroup, EncounterParticipantHealthDatum, EncounterParticipantMetrics, EncounterParticipantTurnData, EncounterRegion, EncounterSession, EncounterSessionTimelineEntry
from application.models.initiative.group import EncounterGroup
from application.models.initiative.participant import Participant, ParticipantGroup, ParticipantHealthDatum
from application.models.initiative.tracked_encounter import TrackedEncounter
from application.models.user import User, Identity, Role, Permission, UserRole
from application.models.profile import Profile
from application.models.entitlement import Entitlement, EntitlementGrant

app = create_app()

migrate = Migrate(app, db)
manager = Manager(app)

# provide a migration utility command
manager.add_command('db', MigrateCommand)

# enable python shell with application context


@manager.shell
def shell_ctx():
    return dict(app=app,
                db=db,
                User=User, Identity=Identity, Role=Role, Profile=Profile, Permission=Permission, UserRole=UserRole,
                Entitlement=Entitlement, EntitlmentGrant=EntitlementGrant,
                GameSystem=GameSystem, GameSystemFacetDatum=GameSystemFacetDatum, GameSystemImageDatum=GameSystemImageDatum,
                Condition=Condition, ConditionHealthAdjustment=ConditionHealthAdjustment,
                Encounter=Encounter, EncounterParticipant=EncounterParticipant, EncounterParticipantGroup=EncounterParticipantGroup, EncounterParticipantHealthDatum=EncounterParticipantHealthDatum, EncounterParticipantMetrics=EncounterParticipantMetrics, EncounterParticipantTurnData=EncounterParticipantTurnData, EncounterRegion=EncounterRegion, EncounterSession=EncounterSession, EncounterSessionTimelineEntry=EncounterSessionTimelineEntry,
                EncounterGroup=EncounterGroup,
                Participant=Participant, ParticipantGroup=ParticipantGroup, ParticipantHealthDatum=ParticipantHealthDatum,
                TrackedEncounter=TrackedEncounter)


if __name__ == '__main__':
    manager.run()
