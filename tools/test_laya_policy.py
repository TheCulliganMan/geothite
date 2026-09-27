import unittest

from laya_policy import Planner, Plan, Action, screen_text


def observation(screen="overworld", text="", name="PlayersHouse2F", point=(1, 1), menus=None):
    rows = [[{"terrain": "Wall", "permission": 7} for _ in range(7)] for _ in range(7)]
    for y in range(1, 6):
        for x in range(1, 6):
            rows[y][x] = {"terrain": "Land", "permission": 0}
    rows[1][5] = {"terrain": "Land", "permission": 0x7a}
    return {"status": {"screen": screen, "party": []},
            "observe": {"text": text, "menus": menus or []},
            "flow_state": {"animating": False},
            "map_info": {"name": name, "player": {"x": point[0], "y": point[1], "facing": "Down"},
                         "objects": [], "terrain": {"origin_x": 0, "origin_y": 0, "rows": rows}}}


class PlannerTests(unittest.TestCase):
    def test_clock_confirmation_does_not_offer_cancel_or_pause(self):
        plan = Planner().plan(observation("clock", "DAY 10 o'clock? > YES / NO"))
        self.assertEqual([a.button for a in plan.actions.values() if a.allowed], ["a"])
        self.assertNotIn("terrain", plan.context)

    def test_unnecessary_menu_is_closed(self):
        plan = Planner().plan(observation(text="START > PACK", menus=[{"kind": "start"}]))
        self.assertEqual(plan.actions["back"].button, "b")

    def test_route_goes_around_wall(self):
        obs = observation()
        obs["map_info"]["terrain"]["rows"][1][2] = {"terrain": "Wall", "permission": 7}
        plan = Planner().plan(obs)
        self.assertTrue(plan.actions["down"].allowed)
        self.assertFalse(plan.actions["right"].allowed)
        self.assertIn("stairs", plan.objective)

    def test_facing_change_alone_does_not_mark_impassable(self):
        planner = Planner()
        before = observation()
        after = observation()
        after["map_info"]["player"]["facing"] = "Right"
        planner.feedback(before, "right", after)
        self.assertTrue(planner.plan(after).actions["right"].allowed)
        planner.feedback(after, "right", after)
        self.assertFalse(planner.plan(after).actions["right"].allowed)

    def test_objective_survives_dialogue(self):
        planner = Planner()
        first = planner.plan(observation())
        planner.plan(observation(text="Hello!"))
        self.assertEqual(first.objective, planner.plan(observation()).objective)

    def test_shield_reports_actual_override(self):
        plan = Plan("Go", "", {"blocked": Action("up", "Wall", False), "safe": Action("right", "Safe")})
        answer = {"probabilities": {"blocked": .9, "safe": .1}}
        self.assertEqual(plan.select(answer), ("blocked", "safe", "right"))
        self.assertEqual(plan.select(answer, guarded=False), ("blocked", "blocked", "up"))
        with self.assertRaises(ValueError):
            plan.select({"probabilities": {"blocked": float("nan"), "safe": .1}})

    def test_map_transition_discards_old_target(self):
        planner = Planner()
        planner.plan(observation())
        obs = observation(name="PlayersHouse1F", point=(5, 1))
        obs["map_info"]["terrain"]["rows"][5][3] = {"terrain": "Land", "permission": 0x70}
        plan = planner.plan(obs)
        self.assertEqual(planner.target, (3, 5))
        self.assertIn("door", plan.objective)

    def test_current_page_replaces_stale_script_summary(self):
        obs = observation(text="IntroLabel:\nHi, <PLAYER>!\n> YES / NO")
        obs['observe']['visible_dialogue'] = "Choose a Pokemon."
        self.assertEqual(screen_text(obs), "Choose a Pokemon.\n> YES / NO")

    def test_no_wait_candidate_is_ever_offered(self):
        scenarios = [observation(), observation('intro'), observation('clock', 'DAY 10?'),
                     observation('overworld', 'Hello!'), observation('battle', 'FIGHT')]
        for obs in scenarios:
            plan = Planner().plan(obs)
            self.assertTrue(plan.actions)
            self.assertTrue(all(a.button in {'up', 'down', 'left', 'right', 'a', 'b', 'start', 'select'}
                                for a in plan.actions.values()))
            self.assertNotIn('wait', plan.question()['action']['criteria'])

    def test_observed_exit_destination_beats_nearer_wrong_door(self):
        obs = observation(name='NewBarkTown')
        obs['map_info']['terrain']['rows'][2][1] = {'terrain':'Land', 'permission':0x71}
        obs['map_info']['curriculum_exits'] = [
            {'x':1,'y':2,'target':'PLAYERS_HOUSE_1F'}, {'x':5,'y':1,'target':'ELMS_LAB'}]
        planner = Planner()
        planner.plan(obs)
        self.assertEqual(planner.target, (5, 1))

    def test_connection_without_coordinates_crosses_map_boundary(self):
        obs = observation(name='NewBarkTown', point=(0, 1))
        obs['status']['party'] = [{'nickname':'Cyndaquil'}]
        obs['map_info']['dimensions'] = [7, 7]
        obs['map_info']['terrain']['rows'][1][0] = {'terrain':'Land', 'permission':0}
        obs['map_info']['curriculum_exits'] = [{'kind':'connection','direction':'west','target':'Route29'}]
        plan = Planner().plan(obs)
        self.assertEqual(plan.actions['left'].button, 'left')

    def test_dialogue_takes_precedence_over_background_naming_state(self):
        obs = observation('naming', 'Old script summary')
        obs['observe']['visible_dialogue'] = 'Take good care of your Pokemon!'
        obs['observe']['menus'] = [{'kind':'name_choices','selected':0}]
        plan = Planner().plan(obs)
        self.assertIn('advance', plan.actions)

    def test_animation_is_not_a_model_decision(self):
        obs = observation()
        obs['flow_state']['animating'] = True
        self.assertEqual(Planner().plan(obs).actions, {})


if __name__ == "__main__":
    unittest.main()
