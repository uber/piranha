import json
from typing import Dict, List, Set

import attr
from polyglot_piranha import Filter, OutgoingEdges, Rule, RuleGraph


@attr.s(eq=False)
class RawFilter:
    id = attr.ib(type=str, default="0")
    enclosing_node = attr.ib(type=str, default=None)
    not_enclosing_node = attr.ib(type=str, default=None)
    not_contains = attr.ib(type=List[str], default=[])
    contains = attr.ib(type=str, default=None)
    at_least = attr.ib(type=int, default=1)
    at_most = attr.ib(type=int, default=4294967295)  # u32::MAX
    child_count = attr.ib(type=int, default=4294967295)  # u32::MAX
    sibling_count = attr.ib(type=int, default=4294967295)  # u32::MAX

    def to_toml(self):
        str_reps = []
        if self.enclosing_node:
            str_reps.append(f'enclosing_node = """{self.enclosing_node}"""')
        if self.not_enclosing_node:
            str_reps.append(f'not_enclosing_node = """{self.not_enclosing_node}"""')
        if self.not_contains:
            str_reps.append(f"not_contains = {json.dumps(self.not_contains)}")
        if self.contains:
            str_reps.append(f'contains = """{self.contains}"""')
        if self.at_least != 1:
            str_reps.append(f"at_least = {self.at_least}")
        if self.at_most != 4294967295:
            str_reps.append(f"at_most = {self.at_most}")
        if self.child_count != 4294967295:
            str_reps.append(f"child_count = {self.child_count}")
        if self.sibling_count != 4294967295:
            str_reps.append(f"sibling_count = {self.sibling_count}")

        return "\n".join([f"[[rules.filters]]"] + str_reps)

    def to_filter(self):
        return Filter(
            enclosing_node=self.enclosing_node,
            not_enclosing_node=self.not_enclosing_node,
            not_contains=self.not_contains,
            contains=self.contains,
            at_least=self.at_least,
            at_most=self.at_most,
            # child_count=self.child_count,
            # sibling_count=self.sibling_count,
        )

    @staticmethod
    def from_toml(toml_dict) -> "RawFilter":
        return RawFilter(
            enclosing_node=toml_dict.get("enclosing_node", None),
            not_enclosing_node=toml_dict.get("not_enclosing_node", None),
            not_contains=toml_dict.get("not_contains", []),
            contains=toml_dict.get("contains", None),
            at_least=toml_dict.get("at_least", 1),
            at_most=toml_dict.get("at_most", 4294967295),
            child_count=toml_dict.get("child_count", 4294967295),
            sibling_count=toml_dict.get("sibling_count", 4294967295),
        )


@attr.s(eq=False)
class RawRule:
    name = attr.ib(type=str)
    query = attr.ib(type=str, default=None)
    replace_node = attr.ib(type=str, default=None)
    replace = attr.ib(type=str, default=None)
    groups = attr.ib(type=Set[str], default=set())
    holes = attr.ib(type=Set[str], default=set())
    filters = attr.ib(type=Set[RawFilter], default=set())
    is_seed_rule = attr.ib(type=bool, default=True)

    @staticmethod
    def from_toml(toml_dict) -> "RawRule":
        return RawRule(
            name=toml_dict["name"],
            query=toml_dict.get("query", None),
            replace_node=toml_dict.get("replace_node", None),
            replace=toml_dict.get("replace", None),
            groups=set(toml_dict.get("groups", set())),
            holes=set(toml_dict.get("holes", set())),
            filters=set([RawFilter.from_toml(f) for f in toml_dict.get("filters", [])]),
            is_seed_rule=toml_dict.get("is_seed_rule", True),
        )

    def to_toml(self):
        str_reprs = [f'name = "{self.name}"']
        if self.query:
            str_reprs.append(f'query = """{self.query}"""')
        if self.replace_node:
            str_reprs.append(f'replace_node = "{self.replace_node}"')
        if self.replace:
            str_reprs.append(f'replace = """{self.replace}"""')
        if self.groups:
            str_reprs.append(f"groups = {json.dumps(self.groups)}")
        if self.holes:
            str_reprs.append(f"holes = {json.dumps(self.holes)}")
        if self.filters:
            str_reprs.append(self.filters_to_toml())
        if not self.is_seed_rule:
            str_reprs.append(f"is_seed_rule = {str(self.is_seed_rule).lower()}")
        return "[[rules]]\n" + "\n".join(str_reprs)

    def filters_to_toml(self):
        return "\n".join([filter.to_toml() for filter in self.filters])

    def to_rule(self):
        return Rule(
            name=self.name,
            query=self.query,
            replace_node=self.replace_node,
            replace=self.replace,
            groups=self.groups,
            holes=self.holes,
            filters=set([f.to_filter() for f in self.filters]),
            is_seed_rule=self.is_seed_rule,
        )


@attr.s
class RawRuleGraph:
    rules = attr.ib(type=List[RawRule])
    edges = attr.ib(type=List[Dict])

    def to_toml(self):
        rules_str = "\n\n".join(rule.to_toml() for rule in self.rules)
        edges_str = "\n\n"
        if self.edges:
            edges_str = "\n\n".join(self.edge_to_toml(edge) for edge in self.edges)
        return f"{rules_str}\n{edges_str}"

    @staticmethod
    def edge_to_toml(edge_dict) -> str:
        return "\n".join(
            [
                "[[edges]]",
                f'scope = "{edge_dict["scope"]}"',
                f'from = "{edge_dict["from"]}"',
                f"to = {json.dumps(edge_dict['to'])}",
            ]
        )

    @staticmethod
    def from_toml(toml_dict) -> "RawRuleGraph":
        rules = []
        for toml_rule in toml_dict["rules"]:
            rule = RawRule.from_toml(toml_rule)
            rules.append(rule)
        edges = toml_dict.get("edges", [])
        return RawRuleGraph(rules=rules, edges=edges)

    def to_graph(self):
        return RuleGraph(
            [rule.to_rule() for rule in self.rules],
            [
                OutgoingEdges(edge["from"], edge["to"], edge["scope"])
                for edge in self.edges
            ],
        )
