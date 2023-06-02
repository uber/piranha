# Dataflow analysis for Piranha

This folder implements a small data flow analysis framework `df.rs`.

There's a partial implementation to a "Definite Assignment Analysis" under `basic_analysis.rs`. 
The idea is to track all the variables defined through the rule graph,
and make sure that all the variables are defined before they are used.
Unlike "Definite Assignment Analysis", it tracks "tags" as they propagate
through rule in the `RuleGraph`. 
The goal of the analysis is to find, for each point in the query,
the set of tags that could reach that point without any redefinitions.
The result can then be used to check if the query contains any tag that was not reached.
