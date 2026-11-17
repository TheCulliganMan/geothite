#!/usr/bin/env python3
"""Export Northstar's authored dialogue as a reading script on stdout."""

import argparse
import json
import re
import runpy
import textwrap
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[1] / "planning/minnesota/story.json"

# The source keeps these lines for older scene installers. The current
# receiver and conductor have their own branches and no longer call them.
REPLACED_SOURCE_PASSAGES = {
    "MnBoundaryWatersSurveyShelter": {
        "MN_SURVEY_CHIEFText",
        "MN_BOUNDARY_SURVEY_RECEIVERText",
        "MnBoundarySurveyReceiverRestoreText",
        "MnNorthstarRelayFeedText",
        "MnNorthstarRelayCutText",
    },
    "MnStPaulUnionDepot": {"MN_ST_PAUL_GLACIER_CONDUCTORText"},
}

# Optional visits after the intake. The player can make them in any order.
RETURN_VISITS = (
    ("Winona — Mom after the rescue", "MnWinonaHome1F", "MnWGuardianSurveyReturnText"),
    ("Winona — the nest after the rescue", "MnWinonaFieldStation",
     "MnWTamarackGuideRestoredText MnArcTamarackAfterText MnWTamarackSurveyReturnText"),
    ("Rochester — Gillpup after the rescue", "MnRochesterPokecenter1F", "MnArcHabitatAfterText"),
    ("Minneapolis — Ellis and Jo after the rescue", "MnMinneapolisDispatcherHouse",
     "MnArcEllisAfterText MnMPJoSurveyReturnText"),
    ("St. Paul — Mina after the rescue", "MnStPaulUnionDepot", "MnArcMinaAfterText"),
    ("Iron Range — Nell after the rescue", "MnR09Cave1F", "MnArcNellAfterText"),
    ("Duluth — the channel after the rescue", "MnDuluthHarborWorkshop", "MnHarborSurveyReturnText"),
)

# Editorial reading order. Each entry is one scene or beat, not a runtime trace.
SCENES = (
    ("Winona — downstairs", "MnWinonaHome1F",
     "MnWHomeIntroText MnWHomeGearReceivedText"),
    ("Winona — choosing a partner", "MnWinonaFieldStation",
     "MnWChooseText MnOverhaulStarterReceivedText MnWReadingsText MnWPokedexReceivedText"),
    ("Winona — Reed brings in the case", "MnWinonaFieldStation",
     "MnNSEReedArrivalText MnNSEReedCaseText MnNSETamarackCaseText MnNSEReedDismissText "
     "MnNSETamarackWarmText MnNSEReedChallengeText MnNSEReedWonText MnNSEReedAfterText "
     "MnNSETamarackQuestText MnNSETamarackRestText"),
    ("Rochester — Ivy takes the delivery", "MnRochesterPokecenter1F",
     "MnRCDeliveryText"),
    ("Rochester — the clinic basin", "MnRochesterPokecenter1F",
     "MnArcHabitatText"),
    ("Rochester — Reed's next shift", "MnRochesterPokecenter1F",
     "MnArcRCPathsText MnArcRCChallengeText MnArcRCBeforeText MnArcRCWonText"),
    ("Winona — returning for the egg", "MnWinonaFieldStation",
     "MnWTamarackDeliveryReturnText MnNSEEggOfferText MnNSEEggGivenText MnNSEEggAfterText"),
    ("Mankato — the notebook", "MnMankato", "MnMKNotebookText"),
    ("Minneapolis — Jo at home", "MnMinneapolisDispatcherHouse",
     "MN_MP_DISPATCHER_PARTNERText"),
    ("Minneapolis — ICE hires Reed", "MnMinneapolisDispatchOffice",
     "MnNICECaptainOfferText MnNICEReedQuestionText MnNICEEllisObjectsText MnNICECaptainAnswerText "
     "MnNICEReedAcceptText MnNICECaptainBadgeText MnNICEReedPlayerText MnNICEEllisCallText"),
    ("Minneapolis — the cargo basement", "MnIceCargoBasement",
     "MnNICECargoUmaText MnNICEBreakerText MnNICEBreakerDoneText"),
    ("Minneapolis — cutting power to the cell", "MnIceRelayFloor",
     "MnNICEReedSeenText MnNICEReedWonText MnNICEHoldingText MnNICEHoldingDoneText"),
    ("Minneapolis — getting Ellis out", "MnMinneapolisDispatchOffice",
     "MnMPICESeenText MnMPICEWonText MnMPCellReadyText MnMPCellReleaseText"),
    ("Minneapolis — the broken coupling", "MnMinneapolisDispatchOffice", "MnMPDispatchReadText"),
    ("Minneapolis — checking in on Jo and Ellis", "MnMinneapolisDispatcherHouse",
     "MN_MP_DISPATCHER_PARTNERAfterText MN_MP_RESCUED_DISPATCHERText"),
    ("St. Paul — water in the cut", "MnStPaulUnionDepot",
     "MnNSTPMinaInterruptText MnSPCourierSeenText MnSPCourierWonText"),
    ("St. Paul — Reed wants a turn on the drill", "MnStPaulUnionDepot",
     "MnArcSPBeforeText MnArcSPReadyText MnArcSPBattleText MnArcSPWonText"),
    ("St. Paul — Mina sends word to Brainerd", "MnStPaulUnionDepot",
     "MnNSTPMinaReadyText MnNSTPMinaQuestText"),
    ("St. Cloud — Sera asks for help", "MnStCloud",
     "MnArcSCBeforeText MnArcSCChoiceText MnArcSCStandText MnArcSCReleasedText MnArcSCAfterText"),
    ("Brainerd — the ranger", "MnBrainerd",
     "MnBrainerdRangerBeforeText MN_BRAINERD_RANGERText MnBrainerdRangerGiftText"),
    ("Lake Itasca — the water mark", "MnLakeItasca",
     "MN_ITASCA_MARKERText"),
    ("Bemidji — comparing readings", "MnBemidji",
     "MnBemidjiComparisonText MnBemidjiNotebookMatchText MnBemidjiReportAfterText"),
    ("Iron Range — at the mine entrance", "MnR09Cave1F",
     "MnIronRangeForemanSeenText MnIronRangeForemanWonText MnNDrillScoutQuestText"),
    ("Iron Range — opening the core stairs", "MnIronPumpGallery",
     "MnNDrillGalleryText MnNDrillGalleryDoneText"),
    ("Iron Range — the crew below", "MnIronDrillCore",
     "MnNDrillCrewCallText MnNDrillForemanText MnNDrillReedWorkText "
     "MnNDrillReedSeenText MnNDrillReedWonText MnNDrillReedAfterText MnNDrillPowerText MnNDrillPowerDoneText "
     "MnNDrillCrewReadyText MnNDrillMeterQuestText"),
    ("Grand Rapids — the depot reads the meter", "MnGrandRapidsSurveyDepot",
     "MnGrandRapidsDepotOfferText MnGrandRapidsDepotReadText"),
    ("Duluth — the engineer checks the log", "MnDuluthHarborWorkshop",
     "MN_HARBOR_RECEIVERText MnOverhaulHarborEvidenceText MnOverhaulHarborReactionText "
     "MnOverhaulHarborDirectionsText MnArcHarborIntakeText"),
    ("Grand Marais — a boat outside the rocks", "MnGrandMarais",
     "MnGrandMaraisHarborMasterText MnGrandMaraisHarborChildText"),
    ("Grand Marais — the part", "MnGrandMaraisFieldStation", "MnRewardMachinePartGivenText"),
    ("Grand Marais — the light turns", "MnGrandMaraisLighthouse", "MnGrandMaraisSignalFixedText"),
    ("Grand Marais — back at the harbor", "MnGrandMarais",
     "MnGrandMaraisHarborMasterSignalText MnGrandMaraisHarborChildSignalText"),
    ("Boundary Waters — Hale and the radio", "MnBoundaryWatersSurveyShelter",
     "MnBoundarySurveyChiefSeenText MnBoundarySurveyChiefWonText MnArcLinkReadyText MnArcLinkText"),
    ("Boundary Waters — Reed at the intake", "MnBoundaryIntake",
     "MnArcIntakeSeenText MnVerdigrimWarningText MnVerdigrimOrderText MnVerdigrimExposedText "
     "MnVerdigrimGreenText MnVerdigrimCounterText MnArcFinalReadyText MnArcFinalBeforeText "
     "MnArcFinalWonText MnArcIntakeFinnText"),
    ("Boundary Waters — restoring the flow", "MnBoundaryIntake",
     "MnVerdigrimDrillAskText MnVerdigrimDrillDoneText MnVerdigrimOutletAskText MnVerdigrimOutletDoneText "
     "MnVerdigrimBypassAskText MnVerdigrimBypassDoneText MnVerdigrimRetreatText "
     "MnArcIntakeOpenText MnArcIntakeEscapeText MnArcIntakeCrewText MnArcIntakeReedExitText MnArcIntakeDoneText"),
    ("St. Cloud — Reed after the rescue", "MnStCloudStation", "MnArcAftermathText"),
    *RETURN_VISITS,
    ("Ely — registering for the League", "MnLeagueLobby", "MnElyRegistrationText"),
    *((f"League — room {n}", f"MnLeagueRoom{n}",
       f"MnLeagueRoom{n}BeforeText MnLeagueRoom{n}WonText MnLeagueRoom{n}AfterText")
      for n in range(1, 5)),
    ("League — Casey", "MnLeagueChampion",
     "MnLeagueChampionBeforeText MnLeagueChampionWonText MnLeagueChampionAfterText"),
    ("League — recording the team", "MnLeagueHallOfFame", "MnElyRecordText"),
    ("Winona — home again", "MnWinonaHome2F", "MnWinonaCousinChampionText"),
    ("Winona — back at the field station", "MnWinonaFieldStation", "MnWTamarackGuideChampionText"),
)

INSTALLED_OPENING = (
    ("Winona — downstairs", "MnWinonaHome1F", "MnWHomeIntroText"),
    ("Winona — choosing a partner", "MnWinonaFieldStation", "MnWChooseText MnWReadingsText"),
)

LOSS_LABELS = {
    "MnArcRCWonText": "MnArcRCBattleLostText",
    "MnArcSPWonText": "MnArcSPBattleLostText",
    "MnArcFinalWonText": "MnArcFinalBattleLostText",
}

STARTER_OPTIONS = (
    ("Pouchid", "MN_W_CHIKORITAText"),
    ("Kindlit", "MN_W_CYNDAQUILText"),
    ("Birchit", "MN_W_TOTODILEText"),
)
STARTER_REPLIES = (
    ("If you turn down an offer", "MnWDeclineText"),
    ("If your team is full", "MnWGiftFullText"),
)
INLINE_CHOICES = {
    "MnWChooseText": (*STARTER_OPTIONS, *STARTER_REPLIES),
    "MnNSEEggOfferText": (
        ("If you decline the egg", "MnNSEEggDeclineText"),
        ("If you accept with a full team", "MnNSEEggFullText"),
    ),
    "MnArcRCChallengeText": (("If you decline the match", "MnArcRCDeclinedText"),),
    "MnMPCellReadyText": (("If you leave the latch alone", "MnMPCellDeclinedText"),),
    "MnArcSPReadyText": (("If you decline the match", "MnArcSPDeclinedText"),),
    "MnArcSCChoiceText": (("If you leave", "MnArcSCDeclinedText"),),
    "MnArcLinkReadyText": (
        ("If you put the mic down", "MnBoundarySurveyReceiverDeclinedText"),
    ),
    "MnArcFinalReadyText": (("If you decline the match", "MnArcFinalDeclinedText"),),
    "MnGrandRapidsDepotOfferText": (
        ("If you leave with the meter", "MnGrandRapidsDepotDeclinedText"),
    ),
    "MnGrandMaraisSignalFixedText": (
        ("If your Pack is full when you bring the part", "MnGrandMaraisSignalBagFullText"),
        ("If you return with room for the held reward", "MnGrandMaraisSignalRewardText"),
        ("If you speak to the keeper after receiving the reward", "MnGrandMaraisSignalDoneText"),
    ),
    "MnElyRegistrationText": (
        ("If you put off signing in", "MnElyRegistrationDeclinedText"),
    ),
    **{
        f"{room}BeforeText": (
            ("On a return as Champion", f"{room}RematchText"),
        )
        for room in (*(f"MnLeagueRoom{n}" for n in range(1, 5)), "MnLeagueChampion")
    },
}

ENTRY_ALTERNATIVES = {
    "MnMKNotebookText": (
        ("If you haven't earned the Care Badge", "MnMKNotebookDeniedText"),
        ("If you've already copied the notebook", "MnMKNotebookAfterText"),
    ),
    "MnNICEHoldingText": (
        ("If the shutter is still locked or Reed is still guarding the relay", "MnNICEHoldingDeniedText"),
    ),
    "MnMPCellReadyText": (
        ("If you try the cell before defeating the guard", "MnMPCellLockedText"),
    ),
    "MnGrandRapidsDepotOfferText": (
        ("If you arrive before recovering the meter", "MnGrandRapidsDepotMissingText"),
    ),
    "MN_HARBOR_RECEIVERText": (
        ("If the depot hasn't read the mine meter", "MnOverhaulHarborMissingLogText"),
    ),
    "MnGrandMaraisSignalFixedText": (
        ("If you speak to the keeper without the part", "MnGrandMaraisSignalMissingText"),
    ),
    "MnArcFinalReadyText": (
        ("If Reed won't let you near the switch yet", "MnArcFinalLockedText"),
    ),
    "MnIronRangeForemanSeenText": (
        ("If you arrive without the Pine Badge", "MnIronRangeForemanBadgeDeniedText"),
        ("If you have the badge but lack the signed Bemidji report", "MnIronRangeForemanReportDeniedText"),
    ),
    "MnBoundarySurveyChiefSeenText": (
        ("If you arrive without all eight badges", "MnBoundarySurveyChiefBadgeDeniedText"),
        ("If you have the badges but the harbor signal is still pending", "MnBoundarySurveyChiefSignalDeniedText"),
    ),
    "MnElyRegistrationText": (
        ("If you arrive without all eight badges", "MnElyBadgeDeniedText"),
        ("If you have all eight badges but the intake rescue is incomplete", "MnElySurveyDeniedText"),
    ),
    **{
        f"{room}BeforeText": (
            ("If you haven't qualified for this match on the current run", f"{room}NoRunText"),
        )
        for room in (*(f"MnLeagueRoom{n}" for n in range(1, 5)), "MnLeagueChampion")
    },
    "MnElyRecordText": (
        ("If you haven't won all five matches on the current run", "MnElyRecordDeniedText"),
    ),
}

SPEAKER = re.compile(r"(?:^|(?<=[.!?] ))((?!YES:|NO:)[A-Z][A-Z ]{1,24}): (?=\S)")
# Narrative beats between speeches. These are reading cues, not engine commands.
AFTER_PASSAGE = {
    "MnOverhaulTamarackIntroText": "Choose a partner.",
    "MnWChooseText": "After you accept a partner.",
    "MnNSEReedChallengeText": "Battle Reed.",
    "MnArcRCChallengeText": "If you accept Reed's challenge:",
    "MnArcRCBeforeText": "Battle Reed.",
    "MnNSEEggOfferText": "If you accept the egg and have room on your team:",
    "MnNICEBreakerText": "Switch off the stair lock and go to the relay floor.",
    "MnNICEReedSeenText": "Battle Reed.",
    "MnNICEHoldingText": "Cut power to the cell.",
    "MnMPICESeenText": "Battle the ICE guard.",
    "MnMPCellReadyText": "Open the cell.",
    "MnSPCourierSeenText": "Battle the Lode miner.",
    "MnArcSPReadyText": "Accept Reed's challenge.",
    "MnArcSPBattleText": "Battle Reed.",
    "MnNSTPMinaReadyText": "Agree to ask the ranger about the first pump test.",
    "MnArcSCChoiceText": "Stand beside Sera.",
    "MnIronRangeForemanSeenText": "Battle the foreman.",
    "MnNDrillGalleryText": "Cut power to the stair lock and take the stairs.",
    "MnNDrillReedSeenText": "Battle Reed.",
    "MnNDrillPowerText": "Stop the drill.",
    "MnNDrillCrewReadyText": "Agree to take the meter.",
    "MnGrandRapidsDepotOfferText": "Let the technician read the meter.",
    "MnBoundarySurveyChiefSeenText": "Battle Hale.",
    "MnArcLinkReadyText": "Agree to read the log over the radio.",
    "MnArcFinalReadyText": "Accept Reed's challenge.",
    "MnArcFinalBeforeText": "Battle Reed.",
    "MnVerdigrimDrillAskText": "Stop the drill.",
    "MnVerdigrimOutletAskText": "Close the sump outlet.",
    "MnVerdigrimBypassAskText": "Open the clean bypass.",
    "MnElyRegistrationText": "Sign in and enter the League.",
    **{f"MnLeagueRoom{n}BeforeText": "Battle." for n in range(1, 5)},
    "MnLeagueChampionBeforeText": "Battle Casey.",
    "MnElyRecordText": "The Hall of Fame records the player's team.",
}

BEFORE_PASSAGE = {
    "MnGrandMaraisHarborMasterText": "If you stop in Grand Marais to help with the lighthouse.",
    "MnRewardMachinePartGivenText": "After Tamarack hands over the part.",
    "MnWTamarackDeliveryReturnText": "If you return to the field station after delivering the envelope:",
    "MnNSEEggAfterText": "If you return after taking the egg and before the next story milestone:",
    "MnArcSCAfterText": "If you speak to Sera again after helping her:",
    "MnArcAftermathText": "If you return to the station after the rescue:",
    "MnMPDispatchReadText": "Inspect Ellis's desk after freeing him.",
    "MN_MP_DISPATCHER_PARTNERAfterText": "If you visit Jo after freeing Ellis.",
    "MN_MP_RESCUED_DISPATCHERText": "If you speak to Ellis at home.",
    "MnNICEEllisAfterText": "If you speak to Ellis again after hearing about the pump.",
    "MnBrainerdRangerBeforeText": "Before earning the Lake Badge:",
    "MN_BRAINERD_RANGERText": "After earning the Lake Badge:",
    "MnBrainerdRangerGiftText": "If you don't already have Surf and it fits in your pack:",
    "MnBemidjiNotebookMatchText": "If you brought the Mankato copy:",
    "MnBemidjiReportAfterText": "Either way:",
}

OUTCOME = re.compile(r"(Seen|Won|Lost)(?:Text)?$")
OUTCOMES = {
    "Seen": "Challenge",
    "Won": "Player wins",
    "Lost": "Player loses",
}
TRAINER_TEXT_FIELDS = ("seen_text", "win_text", "loss_text")
FISHING_GURU_REPLIES = {
    "GuruText": "Fishing Guru — first visit",
    "GuruFullText": "Fishing Guru — Key Items pocket full",
    "GuruAfterText": "Fishing Guru — after receiving the rod",
}


def fishing_guru_heading(location, label):
    if location.endswith("FishingGuruHouse") and label.startswith(location):
        return FISHING_GURU_REPLIES.get(label[len(location):])
    return None


def readable_name(identifier):
    identifier = re.sub(r"^(?:Mn|MN_)", "", identifier)
    identifier = re.sub(r"(?:Interior)?Text$", "", identifier)
    identifier = re.sub(r"^Falls([a-z]+)_lines$", r"Falls \1", identifier)
    identifier = re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", " ", identifier)
    identifier = re.sub(r"(?<=[a-z])(?=[A-Z])", " ", identifier)
    identifier = re.sub(r"(?<=[a-z])(?=[0-9])", " ", identifier)
    return " ".join(identifier.replace("_", " ").split())


def passage_prose(pages):
    return " ".join(line.strip() for page in pages for line in page)


def print_passage(pages):
    # Native rows split words, sentences and speeches across dialogue boxes.
    prose = passage_prose(pages)
    for part in SPEAKER.sub(r"\n\n\1: ", prose).strip().split("\n\n"):
        print(textwrap.fill(part, width=84))
        print()


def print_inline_choices(passages, choices):
    shown = set()
    for heading, label in choices:
        if label in passages:
            print(f"    [{heading}]\n")
            print_passage(passages[label])
            shown.add(label)
    return shown


def print_reference(texts, staged, map_filter, trainer_names, repeat_lines):
    print("DIALOGUE BY LOCATION\n")
    print("Entries here are independent encounters, props, and alternate replies.")
    print("They may be reached in different orders.\n")
    for location, passages in texts.items():
        if map_filter and map_filter.casefold() not in location.casefold():
            continue
        remaining = {label: pages for label, pages in passages.items()
                     if (location, label) not in staged}
        if not remaining:
            continue
        print(f"=== {readable_name(location)} ===\n")
        if location.endswith("FishingGuruHouse"):
            for suffix, heading in FISHING_GURU_REPLIES.items():
                pages = remaining.pop(location + suffix, None)
                if pages:
                    print(f"[{heading}]")
                    print_passage(pages)
        # The optional rematch installer shares these three replies across
        # every rematch trainer in the room. They are separate from first
        # battles and from the original conversation that precedes them.
        for suffix, heading in (
            ("Intro", "Optional trainer rematch — Challenge"),
            ("Win", "Optional trainer rematch — Player wins"),
            ("Loss", "Optional trainer rematch — Player loses"),
        ):
            pages = remaining.pop(f"GeothiteRematch_{location}_{suffix}Text", None)
            if pages:
                print(f"[{heading}]")
                print_passage(pages)
        groups = {}
        trainer_groups = {
            label[:outcome.start()]
            for label in remaining
            if (outcome := OUTCOME.search(label))
        }
        for label, pages in remaining.items():
            outcome = OUTCOME.search(label)
            name = label[:outcome.start()] if outcome else label
            branch = outcome[1] if outcome else None
            if not outcome:
                repeat_group = repeat_lines.get((location, label))
                if repeat_group:
                    name = repeat_group
                    branch = "After"
                elif label.endswith("AfterText") and label[:-9] in trainer_groups:
                    name = label[:-9]
                    branch = "After"
            groups.setdefault(name, {}).setdefault(branch, []).append((label, pages))

        for name, branches in groups.items():
            heading = trainer_names.get((location, name), readable_name(name))
            for _, pages in branches.get(None, []):
                print(f"[{heading}]")
                print_passage(pages)
            for branch, caption in {**OUTCOMES, "After": "Later conversation"}.items():
                for label, pages in branches.get(branch, []):
                    title = caption
                    if branch != "After" and (location, label) in repeat_lines:
                        title += " and later conversation"
                    print(f"[{heading} — {title}]")
                    print_passage(pages)


def read_installed_data(pack_path):
    reader = runpy.run_path(str(Path(__file__).with_name("check-ship-floor-source.py")))
    return reader["load"](pack_path)["data"]


def read_installed_maps(pack_path):
    return read_installed_data(pack_path)["maps"]


def installed_texts(pack_path):
    return map_texts(read_installed_maps(pack_path))


def print_redline(before, after, before_pack, after_pack, map_filter, trainer_names):
    print("NORTHSTAR — DIALOGUE REDLINE\n")
    print(f"Before — {before_pack.name}")
    print(f"After — {after_pack.name}\n")
    print("Compares dialogue by location and passage. Pagination-only edits are omitted.\n")
    count = 0
    for location in sorted(before.keys() | after.keys()):
        if map_filter and map_filter.casefold() not in location.casefold():
            continue
        old = before.get(location, {})
        new = after.get(location, {})
        changes = [
            label for label in sorted(old.keys() | new.keys())
            if passage_prose(old.get(label, ())) != passage_prose(new.get(label, ()))
        ]
        if not changes:
            continue
        print(f"=== {readable_name(location)} ===\n")
        for label in changes:
            outcome = OUTCOME.search(label)
            name = label[:outcome.start()] if outcome else label
            heading = fishing_guru_heading(location, label) or trainer_names.get(
                (location, name), readable_name(name)
            )
            if outcome:
                heading += f" — {OUTCOMES[outcome[1]]}"
            print(f"[{heading}]\n")
            for caption, passages in (("BEFORE", old), ("AFTER", new)):
                print(caption)
                if label in passages:
                    print_passage(passages[label])
                else:
                    print("[No passage in this export.]\n")
        count += len(changes)
    print(f"{count} changed passages.")


def trainer_requests(room):
    yield from room["trainer_scripts"].values()
    yield from (battle["request"] for battle in room["scripted_trainer_battles"])


def trainer_reference(maps, trainers):
    names = {}
    repeat_lines = {}
    for location, room in maps.items():
        if not location.startswith("Mn"):
            continue
        for request in trainer_requests(room):
            trainer = trainers.get(request["trainer_id"])
            if not trainer or not trainer["name"]:
                continue
            for field in TRAINER_TEXT_FIELDS:
                label = request[field]
                outcome = OUTCOME.search(label)
                if outcome:
                    key = (location, label[:outcome.start()])
                    names.setdefault(key, set()).add(trainer["name"])
            callback = room["scripts"].get(request["callback"], [])
            if [row["command"] for row in callback] == ["endifjustbattled", "jumptextfaceplayer"]:
                label = callback[1]["args"][0]
                outcome = OUTCOME.search(request["seen_text"])
                name = request["seen_text"][:outcome.start()] if outcome else label
                repeat_lines.setdefault((location, label), set()).add(name)
    unambiguous = {key: next(iter(choices)) for key, choices in names.items() if len(choices) == 1}
    repeat_groups = {
        key: next(iter(groups)) if len(groups) == 1 else None
        for key, groups in repeat_lines.items()
    }
    return unambiguous, repeat_groups


def referenced_texts(maps, texts):
    """Keep text named by an installed command, including cross-map commands.

    A label shared by multiple maps is kept in each because some commands
    resolve their text context at runtime.
    """
    # The Glacier service builder retains the previous conductor program
    # under an archive label. The conductor event points at the new program.
    referenced_labels = {
        arg
        for room in maps.values()
        for script, commands in room["scripts"].items()
        if not (script.startswith("MnGlacier") and script.endswith("Original"))
        for command in commands
        for arg in command["args"]
        if isinstance(arg, str)
    }
    referenced = {}
    for location, passages in texts.items():
        local_trainer_labels = {
            request[field]
            for request in trainer_requests(maps[location])
            for field in TRAINER_TEXT_FIELDS
        }
        referenced[location] = {
            label: pages for label, pages in passages.items()
            if label in referenced_labels or label in local_trainer_labels
        }
    return referenced


def map_texts(maps):
    texts = {}
    for location, room in maps.items():
        if not location.startswith("Mn"):
            continue
        passages = {}
        for label, body in room["script_text_bodies"].items():
            pages = []
            for row in body["commands"]:
                command = row["command"]
                if command == "done":
                    continue
                if command not in ("text", "line", "para"):
                    raise ValueError(f"{location}/{label}: unsupported text command {command}")
                line = row["args"][0]
                if line.startswith('"') and line.endswith('"'):
                    line = json.loads(line)
                if command == "line" and pages:
                    pages[-1].append(line)
                else:
                    pages.append([line])
            if pages:
                passages[label] = pages
        if passages:
            texts[location] = passages
    if not texts:
        raise ValueError("pack has no installed Minnesota dialogue")
    return texts


def matching_authored_texts(path, installed):
    with path.open(encoding="utf-8") as source:
        authored = json.load(source)["dialogue_overhaul"]["texts"]
    return {
        location: {label: pages for label, pages in passages.items()
                   if label in installed[location]}
        for location, passages in authored.items()
        if location in installed
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    inputs = parser.add_mutually_exclusive_group()
    inputs.add_argument("--source", type=Path, default=SOURCE)
    inputs.add_argument("--pack", type=Path, help="read installed Minnesota text from a crystalpack")
    parser.add_argument(
        "--trainer-reference-pack", type=Path,
        help="use installed trainer names and follow-up links in an authored dialogue export",
    )
    view = parser.add_mutually_exclusive_group()
    view.add_argument("--campaign", action="store_true", help="only the main reading sequence")
    view.add_argument("--map", help="all authored passages matching a map name")
    parser.add_argument(
        "--redline-from", type=Path,
        help="compare an earlier crystalpack against --pack or the current authored source",
    )
    parser.add_argument(
        "--preview-source", type=Path,
        help="preview matching source dialogue in the --pack scenes without building a pack",
    )
    parser.add_argument(
        "--include-unreferenced", action="store_true",
        help="with --pack, include stored passages without a direct script reference",
    )
    args = parser.parse_args()
    if args.trainer_reference_pack and (args.pack or args.campaign or args.redline_from):
        parser.error("--trainer-reference-pack requires an authored location or full script export")
    if args.include_unreferenced and not args.pack:
        parser.error("--include-unreferenced requires --pack")
    if args.redline_from and args.campaign:
        parser.error("--redline-from compares dialogue by location")
    if args.preview_source and (not args.pack or args.redline_from):
        parser.error("--preview-source requires --pack and cannot be combined with --redline-from")

    if args.redline_from:
        before_data = read_installed_data(args.redline_from)
        before_maps = before_data["maps"]
        before = map_texts(before_maps)
        before_names, _ = trainer_reference(before_maps, before_data["trainers"]["trainers"])
        if not args.include_unreferenced:
            before = referenced_texts(before_maps, before)
        del before_data, before_maps

        if not args.pack:
            # Missing source entries can be supplied by other builders. The
            # writing review compares only passages with an installed counterpart.
            after = matching_authored_texts(args.source, before)
            if args.map and not any(args.map.casefold() in name.casefold() for name in after):
                parser.error(f"no map matches {args.map!r}")
            before = {
                location: {label: before[location][label] for label in passages}
                for location, passages in after.items()
            }
            print("Source comparison. Only matching installed and authored passages are included.")
            print("Differences can include builder overrides. Revised source has not been installed.\n")
            print_redline(before, after, args.redline_from, args.source, args.map, before_names)
            return

        after_data = read_installed_data(args.pack)
        after_maps = after_data["maps"]
        after = map_texts(after_maps)
        after_names, _ = trainer_reference(after_maps, after_data["trainers"]["trainers"])
        if not args.include_unreferenced:
            after = referenced_texts(after_maps, after)
        if args.map and not any(args.map.casefold() in name.casefold() for name in before.keys() | after.keys()):
            parser.error(f"no map matches {args.map!r}")
        print_redline(before, after, args.redline_from, args.pack, args.map, before_names | after_names)
        return

    home_copy = False
    depot_challenge = False
    trainer_names = {}
    repeat_lines = {}
    if args.pack:
        data = read_installed_data(args.pack)
        maps = data["maps"]
        texts = map_texts(maps)
        if not args.campaign:
            trainer_names, repeat_lines = trainer_reference(maps, data["trainers"]["trainers"])
        home = maps.get("MnMinneapolisDispatcherHouse", {}).get("scripts", {})
        home_copy = any(
            row["command"] == "setevent" and row["args"] == ["EVENT_MN_DISPATCH_NOTE"]
            for row in home.get("MN_MP_RESCUED_DISPATCHER", [])
        )
        depot = maps.get("MnStPaulUnionDepot", {}).get("scripts", {})
        depot_challenge = any(
            row["command"] == "iffalse" and row["args"] == ["MnArcSPChoice"]
            for row in depot.get("MN_NSTP_MINA", [])
        )
        harbor = maps.get("MnDuluthHarborWorkshop", {}).get("scripts", {})
        harbor_passages = [
            row["args"][0] for row in harbor.get("MN_HARBOR_RECEIVER", [])
            if row["command"] == "writetext" and row["args"]
        ]
        legacy_harbor = harbor_passages and "MnOverhaulHarborEvidenceText" not in harbor_passages
        keeper = maps.get("MnGrandMaraisLighthouse", {}).get("scripts", {})
        deferred_lighthouse_reward = "MnGrandMaraisSignalReward" in keeper
        if args.preview_source:
            for location, passages in matching_authored_texts(args.preview_source, texts).items():
                if legacy_harbor and location == "MnDuluthHarborWorkshop":
                    passages = {
                        label: pages for label, pages in passages.items()
                        if label not in harbor_passages
                    }
                if location == "MnGrandMaraisLighthouse" and not deferred_lighthouse_reward:
                    passages.pop("MnGrandMaraisSignalBagFullText", None)
                texts[location].update(passages)
    else:
        with args.source.open(encoding="utf-8") as source:
            texts = json.load(source)["dialogue_overhaul"]["texts"]
        if args.trainer_reference_pack:
            reference = read_installed_data(args.trainer_reference_pack)
            trainer_names, repeat_lines = trainer_reference(
                reference["maps"], reference["trainers"]["trainers"]
            )
        if args.source.resolve() == SOURCE:
            for location, labels in REPLACED_SOURCE_PASSAGES.items():
                passages = texts.get(location, {})
                for label in labels:
                    passages.pop(label, None)
    if args.map and not any(args.map.casefold() in name.casefold() for name in texts):
        parser.error(f"no map matches {args.map!r}")

    print("NORTHSTAR — STORY SCRIPT\n")
    if args.pack:
        if args.preview_source:
            print(f"Preview of {args.preview_source.name} using scenes from {args.pack.name}.")
            print("Matching source passages replace pack wording here. Other text is from the pack.")
            print("This preview does not modify the pack or predict later builder overrides.")
        else:
            print(f"Installed dialogue from {args.pack.name}. Scene order is editorial.")
        if args.include_unreferenced:
            print("The location reference includes stored passages without direct script references.")
        else:
            print("The location reference includes passages named by installed scripts and trainer battles.")
        print("References alone do not establish which branches players can reach.\n")
    else:
        print("From the current authored dialogue. Scene order is editorial.")
        if args.trainer_reference_pack:
            print(f"Trainer names and follow-up links come from {args.trainer_reference_pack.name}.")
        print("Some source entries may be superseded. Verify against the installed pack.\n")
    staged = set()
    if not args.map:
        print("CAMPAIGN\n")
        scenes = (*INSTALLED_OPENING, *SCENES[2:]) if args.pack else SCENES
        for title, location, labels in scenes:
            passages = texts.get(location, {})
            is_return_visit = (title, location, labels) in RETURN_VISITS
            if is_return_visit:
                alternatives = labels.split()
                if location == "MnWinonaFieldStation":
                    alternatives = [next((label for label in alternatives if label in passages), "")]
                labels = " ".join(label for label in alternatives if label in passages)
                if not labels:
                    continue
            if args.pack and not depot_challenge and location == "MnStPaulUnionDepot":
                if "MnArcSPBeforeText" in labels:
                    continue
                if "MnNSTPMinaReadyText" in labels and "MnSPManifestText" in passages:
                    labels = "MnSPManifestText " + labels
            if home_copy and labels == "MnMPDispatchReadText":
                title = "Minneapolis — talking to Ellis at home"
                location = "MnMinneapolisDispatcherHouse"
                passages = texts.get(location, {})
                labels = "MnNICEEllisQuestText"
            if location == "MnMinneapolisDispatcherHouse" and "MN_MP_DISPATCHER_PARTNERAfterText" in labels:
                if home_copy:
                    labels = labels.replace("MN_MP_RESCUED_DISPATCHERText", "MnNICEEllisAfterText")
                if args.pack:
                    labels = " ".join(label for label in labels.split() if label in passages)
                    if not labels:
                        continue
            if args.pack and legacy_harbor and location == "MnDuluthHarborWorkshop" and not is_return_visit:
                labels = " ".join(harbor_passages)
            if args.pack and labels in ("MnWinonaCousinChampionText", "MnWTamarackGuideChampionText"):
                if labels not in passages:
                    print(f"=== {title} ===\n\n    [No Champion-specific conversation in this pack.]\n")
                    continue
            print(f"=== {title} ===\n")
            if args.pack and legacy_harbor and location == "MnDuluthHarborWorkshop" and not is_return_visit:
                print("    [This pack uses the earlier harbor conversation.]\n")
                if args.preview_source:
                    print(
                        "    [The harbor briefing below stays in its installed wording. "
                        "The revised briefing needs separate evidence, reaction, and directions pages "
                        "that this scene does not call.]\n"
                    )
            if is_return_visit:
                print("    [Optional visit after the rescue.]\n")
            for label in labels.split():
                if label not in passages:
                    print("    [An expected passage is missing from this input.]\n")
                    continue
                if label in BEFORE_PASSAGE:
                    print(f"    [{BEFORE_PASSAGE[label]}]\n")
                entry_replies = print_inline_choices(passages, ENTRY_ALTERNATIVES.get(label, ()))
                if entry_replies:
                    staged.update((location, reply) for reply in entry_replies)
                    condition = (
                        "When you bring the keeper the part"
                        if label == "MnGrandMaraisSignalFixedText"
                        else "If you meet the entry requirements"
                    )
                    print(f"    [{condition}]\n")
                choices = INLINE_CHOICES.get(label, ())
                if any(choice.endswith("RematchText") and choice in passages for _, choice in choices):
                    print("    [Before becoming Champion]\n")
                outcome = OUTCOME.search(label)
                if outcome and outcome[1] == "Won":
                    loss = LOSS_LABELS.get(label, label[:outcome.start()] + "LostText")
                    if loss in passages:
                        print("    [If you lose the battle.]\n")
                        print_passage(passages[loss])
                        staged.add((location, loss))
                        print("    [If you win, the scene continues.]\n")
                print_passage(passages[label])
                if choices:
                    staged.update(
                        (location, choice)
                        for choice in print_inline_choices(passages, choices)
                    )
                if label in AFTER_PASSAGE:
                    print(f"    [{AFTER_PASSAGE[label]}]\n")
                staged.add((location, label))
    if not args.campaign:
        if args.pack and not args.include_unreferenced:
            texts = referenced_texts(maps, texts)
        print_reference(texts, staged, args.map, trainer_names, repeat_lines)


if __name__ == "__main__":
    main()
