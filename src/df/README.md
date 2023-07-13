# Dataflow analysis for Piranha

This folder implements a small data flow analysis framework `df.rs`.

There's an implementation to a "Definite Assignment Analysis" under `tag_analysis.rs`. 
The idea is to track all the variables defined through the rule graph,
and make sure that all the variables are defined before they are used.
Unlike "Definite Assignment Analysis", it tracks "tags" as they propagate
through rules in the `RuleGraph`. 
The goal of the analysis is to find, for each point in the graph,
the set of tags that will always reach that point.
The result can then be used to check if the query contains any variable tag that was not defined.
