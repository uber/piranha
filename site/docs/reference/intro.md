---
id: intro
title: Overview
sidebar_label: Overview
---

import useBaseUrl from '@docusaurus/useBaseUrl';

Polyglot Piranha is a flexible multilingual structural search/replace engine that allows users to apply chains of interdependent structural search/replace rules for deeper cleanups. Polyglot Piranha builds upon tree-sitter queries for expressing the structural search/replace rules.

<div style={{display: 'flex', justifyContent: 'center'}}>
  <img src={useBaseUrl('/img/piranha_architecture.svg')} alt="Polyglot Piranha Architecture" width="800" height="500"/>
</div>

This is the higher level architecture of Polyglot Piranha.
At its heart, Polyglot Piranha is a structural find/replacement (rewrite) engine and pre-build language specific cleanup rules like - like simplifying boolean expressions, simplifying `if-else` statements, deleting empty class, deleting files with no type declarations, inline local variables, and many more.
A user provides :
- A set (or, a graph) of structural find/replace rules
- Path to the code base
- [Arguments](#piranha-arguments) to modify Piranha's behavior (like deleting associated comments).


