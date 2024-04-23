# Modular Rust Model-Checker (WIP)

Sally will be a modular model checker, able to handle multiple model semantics, apply translation between semantics, compute analytic solutions as well as performing statistical model checking and discrete verification.

It's primary objective is the verification of Time Petri nets, but new semantics can easily be added by implementing traits.

The verification engine will be based on a "Model solving graph", where vertices are semantics, edges are translations between semantics, and analytic solutions are annoted to the corresponding semantics.

Each type of element is described here after :
- Semantics : can be discrete, timed, stochastic, 2 players game, or be the symbolic state space representation of another semantic.
- Translations : link between two semantics, can be a symbolic space computation, a one-to-many translation, a complete one-to-many, or an observation function.
- Solutions : decidable algorithm to compute the solution of a query on a specific semantic, associated with a quantifier (EF, EG, AF, AG,...).

Once registered in the solver, the engine is able to find (if possible) a path for a query, or else defaults to discrete verification or SMC.
