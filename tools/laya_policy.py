"""Ground public Geothite observations into small, explicit Laya decisions.

Like the upstream Snake demo, navigation is planned outside the model. No game
state is mutated here: every executed action is an ordinary joypad button.
"""

from collections import Counter, deque
from dataclasses import dataclass
import math
import re
import heapq


DIRECTIONS = {"up": (0, -1), "down": (0, 1), "left": (-1, 0), "right": (1, 0)}
WARP_DIRECTIONS = {0x70: "down", 0x76: "left", 0x78: "up", 0x7e: "right"}
STAIRS = {0x72, 0x73, 0x7a}


def position(observation):
    p = observation["map_info"]["player"]
    return p["x"], p["y"]


def add(point, direction):
    dx, dy = DIRECTIONS[direction]
    return point[0] + dx, point[1] + dy


def screen_text(observation):
    view = observation["observe"]
    if view.get('battle_message'):
        return view['battle_message']
    if view.get("visible_dialogue"):
        choices = [line for line in view.get("text", "").splitlines() if re.search(r"(?:^| / )\s*>\s", line)]
        return view["visible_dialogue"] + ("\n" + "\n".join(choices) if choices else "")
    return view.get("text", "")


def normalized(name):
    return re.sub(r"[^a-z0-9]", "", name.lower())


@dataclass
class Action:
    button: str
    description: str
    allowed: bool = True


@dataclass
class Plan:
    objective: str
    context: str
    actions: dict[str, Action]

    def question(self):
        return {"action": {
            "type": "choice",
            "instructions": "Choose the safe action that best advances the stated objective.",
            "criteria": {key: action.description for key, action in self.actions.items()},
        }}

    def select(self, answer, guarded=True):
        probabilities = answer["probabilities"]
        if set(probabilities) != set(self.actions) or any(
            not math.isfinite(p) or not 0 <= p <= 1 for p in probabilities.values()
        ):
            raise ValueError("Invalid model probabilities; no input executed")
        proposed = max(probabilities, key=probabilities.get)
        allowed = [key for key, action in self.actions.items() if action.allowed]
        if not allowed:
            raise ValueError("Planner has no permitted action")
        executed = max(allowed, key=probabilities.get) if guarded else proposed
        return proposed, executed, self.actions[executed].button


class Planner:
    def __init__(self):
        self.tiles = {}
        self.visits = Counter()
        self.failed_edges = Counter()
        self.arrivals = {}
        self.previous_map = None
        self.target = None
        self.target_map = None
        self.target_kind = None
        self.transitions = []
        self.recent = deque(maxlen=4)
        self.interacted = set()
        self.pending_interaction = None
        self.goal = "Leave home and visit Professor Elm to receive a Pokemon."

    def remember(self, observation):
        world = observation["map_info"]
        name, point = world["name"], position(observation)
        if name != self.previous_map:
            if self.previous_map is not None:
                self.transitions.append(f"{self.previous_map} -> {name}")
                self.arrivals[name] = point
            self.previous_map = name
            self.target = self.target_map = self.target_kind = None
        terrain = world.get("terrain", {})
        known = self.tiles.setdefault(name, {})
        for dy, row in enumerate(terrain.get("rows", [])):
            for dx, tile in enumerate(row):
                if tile is not None:
                    known[terrain["origin_x"] + dx, terrain["origin_y"] + dy] = tile
        if observation["status"].get("party"):
            self.goal = "Leave the lab and explore west from New Bark Town."

    def feedback(self, before, button, after):
        name, point = before["map_info"]["name"], position(before)
        moved = after["map_info"]["name"] != name or position(after) != point
        self.visits[name, point] += 1
        if button in DIRECTIONS and before["status"]["screen"] == "overworld" and not screen_text(before):
            edge = name, point, button
            # A first press can merely turn. Only repeated failures block an edge.
            self.failed_edges[edge] = 0 if moved else self.failed_edges[edge] + 1
        if self.pending_interaction and after["observe"]["text"]:
            self.interacted.add(self.pending_interaction)
            self.pending_interaction = None
            self.target = None
        outcome = "moved" if moved else ("screen changed" if screen_text(before) != screen_text(after) else "no visible change")
        self.recent.append(f"{button}: {outcome}")

    def simple(self, objective, text, key, button, consequence):
        # All candidates are real buttons. Animation timing is handled by the
        # caller before inference, never represented as a model action.
        if button is None:
            return Plan(objective, text, {})
        alternative_button = "b" if button != "b" else "a"
        key = {'a':'advance', 'b':'back'}.get(button, button)
        alternate_key = 'cancel' if alternative_button == 'b' else 'confirm'
        alternative = Action(alternative_button, 'Cancel using B.' if alternative_button == 'b' else 'Confirm the unrelated selection using A.', False)
        context = f"Goal: {objective}\nScreen: {text[:400]}\nPress {button.upper()}: {consequence}"
        return Plan(objective, context, {
            key: Action(button, consequence), alternate_key: alternative,
        })

    def plan(self, observation):
        self.remember(observation)
        screen = observation["status"]["screen"]
        view = observation["observe"]
        text = screen_text(observation)
        menus = view.get("menus", [])
        if observation.get("flow_state", {}).get("animating"):
            return Plan("Animation in progress.", text, {})
        if view.get('battle_message'):
            # Presentation can outlive core's battle state. Never interpret its
            # ignored directional input as evidence of an impassable map tile.
            for direction in DIRECTIONS:
                self.failed_edges[observation['map_info']['name'], position(observation), direction] = 0
            return self.simple('Acknowledge the visible battle result before moving.', text,
                               'advance_battle_message', 'a', 'Confirm this battle message and continue.')
        if screen == "intro" or text.strip() == "PRESS START":
            return self.simple("Open the game title menu.", text, "begin_game", "start", "Safe. Press Start to open the title menu.")
        if screen == "title":
            lines = text.splitlines()
            desired = "CONTINUE" if "CONTINUE" in text else "NEW GAME"
            highlighted = next((line for line in lines if line.lstrip().startswith(">")), "")
            button = "a" if desired in highlighted else "up"
            return self.simple(f"Select {desired}.", text, "select_game", button, f"Safe. {'Confirm' if button == 'a' else 'Move selection toward'} {desired}.")
        if screen == "naming" and not view.get('visible_dialogue'):
            names = next((m for m in menus if m.get("kind") == "name_choices"), None)
            entry = next((m for m in menus if m.get("kind") == "name_input"), None)
            if names:
                selected = names.get("selected", 0)
                button = "down" if selected == 0 else "a"
                return self.simple("Choose an existing trainer name.", text, "preset_name", button, "Safe. Select a preset name instead of opening the keyboard.")
            if entry:
                button = "start" if entry.get("value") else "a"
                return self.simple("Finish entering the name.", text, "finish_name", button, "Safe. Finish the entered name." if button == 'start' else "Enter the selected letter first.")
        if screen in {"clock", "gender", "introduction"}:
            return self.simple("Accept the setup choice and continue the introduction.", text, "confirm_setup", "a", "Safe. Confirm the highlighted choice or advance dialogue.")
        if any(m.get("kind") in {"start", "pack", "party", "pokedex", "pokegear"} for m in menus) and screen != "battle":
            return self.simple(self.goal, text, "close_menu", "b", "Safe. Close the unrelated menu to resume the objective.")
        if screen == "battle":
            return self.simple("Win the current battle using a move.", text, "battle_confirm", "a", "Confirm the selected Fight command, move, or battle message.")
        if text.strip():
            if observation['map_info']['name'] == 'ElmsLab':
                self.lab_conversation_seen = True
            if 'nickname' in text.lower() and ('YES' in text or 'NO' in text):
                button = 'a' if re.search(r'>\s*NO', text) else 'down'
                return self.simple('Keep the Pokemon species name.', text, 'keep_name', button,
                                   'Safe. Select NO to decline a custom nickname and continue.')
            # No raw terrain is supplied while a dialogue/menu owns input.
            return self.simple("Continue the conversation and accept help.", text, "continue_dialogue", "a", "Safe. Advance this dialogue or accept the highlighted answer.")
        return self.navigation(observation)

    def navigation(self, observation):
        world = observation["map_info"]
        name, point = world["name"], position(observation)
        known = self.tiles[name]
        objects = {(obj["x"], obj["y"]): obj.get("name", "person") for obj in world.get("objects", [])}

        def open_tile(p):
            return p in known and known[p]["terrain"] == "Land" and p not in objects

        def distances(start):
            result, queue = {start: 0}, deque([start])
            while queue:
                here = queue.popleft()
                for direction in DIRECTIONS:
                    there = add(here, direction)
                    if there not in result and open_tile(there) and self.failed_edges[name, here, direction] < 2:
                        result[there] = result[here] + 1
                        queue.append(there)
            return result

        reachable = distances(point)
        party = bool(observation["status"].get("party"))
        onward = {'NewBarkTown': 'Route29', 'Route29': 'CherrygroveCity'}
        desired = onward.get(name) if party else None
        connection = next((e for e in world.get('curriculum_exits', [])
                           if e.get('kind') == 'connection' and normalized(e.get('target', '')) == normalized(desired or '')), None)
        if connection:
            return self.connection_plan(observation, connection, objects)
        targets = []
        if name == "ElmsLab" and not party:
            # Object names and positions are already exposed by the observation bridge.
            people = [(p, label) for p, label in objects.items() if (name, label) not in self.interacted]
            for obj, label in people:
                label_lower = label.lower().replace("elmslab", "")
                if "elm" in label_lower and getattr(self, 'lab_conversation_seen', False):
                    continue
                for direction in DIRECTIONS:
                    p = add(obj, direction)
                    if p in reachable:
                        priority = 0 if "elm" in label_lower else (1 if "ball" in label_lower else 2)
                        targets.append((priority, reachable[p], p, ("person", obj, label)))
        if not targets:
            warps = [(p, tile["permission"]) for p, tile in known.items() if 0x70 <= tile.get("permission", -1) <= 0x7f and p in reachable]
            desired_map = {
                'PlayersHouse2F': 'PlayersHouse1F', 'PlayersHouse1F': 'NewBarkTown',
                'NewBarkTown': 'Route29' if party else 'ElmsLab', 'ElmsLab': 'NewBarkTown',
                'Route29': 'CherrygroveCity',
            }.get(name)
            exits = world.get('curriculum_exits', [])
            desired_exits = [e for e in exits if normalized(e.get('target', '')) == normalized(desired_map or '')]
            if desired_exits:
                warps = [((e['x'], e['y']), known.get((e['x'], e['y']), {}).get('permission', 0))
                         for e in desired_exits if 'x' in e and 'y' in e and (e['x'], e['y']) in reachable]
            elif exits and desired_map:
                warps = []
            connection = next((e for e in desired_exits if e.get('kind') == 'connection'), None)
            if connection:
                direction = {'west':'left', 'east':'right', 'north':'up', 'south':'down'}[connection['direction']]
                width, height = world['dimensions']
                def boundary(p):
                    return {'left':p[0] == 0, 'right':p[0] == width - 1,
                            'up':p[1] == 0, 'down':p[1] == height - 1}[direction]
                targets.extend((0, reachable[p], p, ('connection', direction, connection['target']))
                               for p in reachable if boundary(p))
            for p, permission in warps:
                if p == self.arrivals.get(name):
                    continue
                if name == "PlayersHouse2F":
                    priority = 0 if permission in STAIRS else 2
                elif name == "PlayersHouse1F":
                    priority = 0 if permission not in STAIRS else 3
                elif name == "NewBarkTown" and not party:
                    # Elm's lab is in the northwest of town. Coordinates are
                    # discovered from observed door tiles, not a copied map.
                    priority = p[0] + p[1]
                else:
                    priority = 0
                targets.append((priority, reachable[p], p, ("exit", permission)))
        if name == "NewBarkTown" and party and not targets:
            targets = [(p[0], reachable[p], p, ("explore",)) for p in reachable if p != point]
        if not targets:
            # Expand the observed map toward its frontier, remembering visited tiles.
            for p in reachable:
                if p == point:
                    continue
                frontier = any(add(p, d) not in known for d in DIRECTIONS)
                targets.append((0 if frontier else 1, self.visits[name, p] * 20 + reachable[p], p, ("explore",)))
        if not targets:
            return Plan("No observed route is available. Controller needs a new observation or manual repositioning.", "", {})
        candidates = {entry[2]: entry[3] for entry in targets}
        if self.target not in candidates or self.target_map != name or (self.target == point and self.target_kind[0] == "explore"):
            _, _, self.target, self.target_kind = min(targets)
            self.target_map = name
        target, kind = self.target, self.target_kind
        if kind[0] == "person":
            objective = f"Approach and talk to {kind[2]} to receive a Pokemon."
            if point == target:
                facing = next(d for d in DIRECTIONS if add(point, d) == kind[1])
                button = "a" if world["player"]["facing"].lower() == facing else facing
                if button == "a":
                    self.pending_interaction = name, kind[2]
                return self.simple(objective, f"Adjacent to {kind[2]}.", "talk_to_person", button, f"Safe. {'Speak to' if button == 'a' else 'Face'} {kind[2]}.")
        elif kind[0] == 'connection':
            objective = f"Travel {kind[1]} to {kind[2]} through the map boundary at {target}."
            if point == target:
                return self.simple(objective, 'Standing at the connected map boundary.', 'cross_boundary', kind[1],
                                   'Safe. Cross into the connected map toward the objective.')
        elif kind[0] == "exit":
            objective = f"Reach the {'stairs' if kind[1] in STAIRS else 'door'} at {target} to leave {name}."
            if name == "NewBarkTown":
                objective = f"Enter the northwest building to find Professor Elm. Door at {target}."
            if point == target and kind[1] in WARP_DIRECTIONS:
                button = WARP_DIRECTIONS[kind[1]]
                return self.simple(objective, "Standing on the exit carpet.", "cross_exit", button, "Safe. Step across the exit in its indicated direction.")
        else:
            objective = f"Explore toward {target} to discover a route. {self.goal}"
        current_distance = reachable[target]
        actions = {}
        for direction in DIRECTIONS:
            neighbor = add(point, direction)
            legal = open_tile(neighbor) and self.failed_edges[name, point, direction] < 2
            remaining = distances(neighbor).get(target) if legal else None
            progress = remaining is not None and remaining < current_distance
            if not legal:
                description = "Blocked by observed terrain, an object, or a failed move. Do not choose."
            elif progress:
                description = f"Safe. Best progress toward objective; {remaining} steps remain."
            else:
                description = "Walkable but moves away from the planned objective."
            actions[direction] = Action(direction, description, progress)
        if not any(a.allowed for a in actions.values()):
            self.target = None
            return Plan("Replan after the route became blocked.", "", {})
        context = f"Objective: {objective}\nMap {name}. Position {point}. Route length {current_distance}."
        if self.recent:
            context += "\nRecent outcomes: " + "; ".join(self.recent)
        return Plan(objective, context, actions)

    def connection_plan(self, observation, connection, objects):
        """Plan toward the destination over surveyed and explicitly unknown tiles.

        Unknown future tiles carry a penalty; executed next steps must already
        be observed traversable. This allows purposeful frontier exploration
        without replacing the destination with the nearest arbitrary frontier.
        """
        world = observation['map_info']
        name, point = world['name'], position(observation)
        known = self.tiles[name]
        width, height = world['dimensions']
        direction = {'west':'left','east':'right','north':'up','south':'down'}[connection['direction']]
        destination = connection['target']
        self.goal = f'Reach {destination} through the {connection["direction"]} exit of {name}.'
        self.target = self.target_map = self.target_kind = None

        def traversable(p):
            if not (0 <= p[0] < width and 0 <= p[1] < height) or p in objects:
                return False
            return p not in known or known[p]['terrain'] == 'Land'

        def boundary(p):
            return {'left':p[0] == 0,'right':p[0] == width-1,'up':p[1] == 0,'down':p[1] == height-1}[direction]

        if boundary(point):
            return self.simple(self.goal, f'At map boundary {point}.', 'cross_boundary', direction,
                               f'Cross {connection["direction"]} into {destination}.')
        costs, queue = {}, []
        for x in range(width):
            for y in range(height):
                p = x, y
                if boundary(p) and traversable(p):
                    costs[p] = 0
                    heapq.heappush(queue, (0, p))
        while queue:
            cost, here = heapq.heappop(queue)
            if cost != costs[here]:
                continue
            # Reverse traversal respects each directed failed-movement edge.
            for move, (dx, dy) in DIRECTIONS.items():
                prior = here[0]-dx, here[1]-dy
                if not traversable(prior) or self.failed_edges[name, prior, move] >= 2:
                    continue
                tile = known.get(here)
                unknown = tile is None
                grass = tile is not None and tile.get('permission') in {8, 0x10, 0x14, 0x18, 0x1c, 0x28, 0x48, 0x49, 0x4a, 0x4b, 0x4c}
                step_cost = 4 if unknown else (1.5 if grass else 1)
                candidate = cost + step_cost
                if candidate < costs.get(prior, math.inf):
                    costs[prior] = candidate
                    heapq.heappush(queue, (candidate, prior))
        choices = {}
        for move in DIRECTIONS:
            neighbor = add(point, move)
            tile = known.get(neighbor)
            legal = tile is not None and traversable(neighbor) and self.failed_edges[name, point, move] < 2
            cost = costs.get(neighbor, math.inf)
            choices[move] = neighbor, tile, legal, cost
        best = min((item[3] for item in choices.values() if item[2]), default=math.inf)
        if not math.isfinite(best):
            return Plan(f'No surveyed step leads toward {destination}; manual repositioning is needed.', '', {})
        actions = {}
        for move, (neighbor, tile, legal, remaining) in choices.items():
            if not legal:
                description = f'{move} to {neighbor}: blocked by terrain, object, or confirmed failed movement.'
            elif not math.isfinite(remaining):
                description = f'{move} to {neighbor}: reachable tile but no known or exploratory route to {destination}.'
            else:
                progress = remaining <= best + .01
                description = (f'{move} to {neighbor}: {"best route" if progress else "longer detour"} to {destination}; '
                               f'estimated remaining cost {remaining:g}; visited {self.visits[name, neighbor]} times.')
            actions[move] = Action(move, description, legal and math.isfinite(remaining) and remaining <= best + .01)
        party = observation['status'].get('party', [])
        health = ', '.join(f'{p["nickname"]} HP {p["hp"]}/{p["max_hp"]}' for p in party)
        context = (f'Objective: {self.goal}\nAt {point}, facing {world["player"]["facing"]}. {health}. '
                   'Avoid walls and unnecessary grass encounters. Route estimates penalize unexplored tiles. '
                   'Only the next surveyed step will execute; then observe again.')
        if self.recent:
            context += '\nLast outcomes: ' + '; '.join(self.recent)
        return Plan(self.goal, context, actions)


def verify_prompt(model, plan):
    """Fail rather than silently truncate the state or candidate descriptions."""
    from laya_coreml.common import build_prefix
    question = plan.question()
    internal = model._to_internal(question["action"])
    prefix, _ = build_prefix(model.tok, internal, model.cfg.get("head_max_len", 192))
    state_tokens = model.tok(plan.context, add_special_tokens=False)["input_ids"]
    if len(prefix) + len(state_tokens) + 1 > model.cfg.get("max_len", 512):
        raise ValueError("Planner context exceeds the model token budget")
    return question
