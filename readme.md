# Modular Rust Model-Checker (WIP)

Sally will be a modular model checker, able to handle multiple model semantics, apply translation between semantics, compute analytic solutions as well as performing statistical model checking and discrete verification.

It's primary objective is the verification of Time Petri nets, but new semantics can easily be added by implementing traits.

The verification engine will be based on a "Model solving graph", where vertices are semantics, edges are translations between semantics, and analytic solutions are annoted to the corresponding semantics.

Each type of element is described here after :
- Semantics : can be discrete, timed, stochastic, 2 players game, or be the symbolic state space representation of another semantic.
- Translations : link between two semantics, can be a symbolic space computation, a one-to-many translation, a complete one-to-many, or an observation function.
- Solutions : decidable algorithm to compute the solution of a query on specific semantics, associated with a quantifier (EF, EG, AF, AG,...).

Once registered in the solver, the engine is able to find (if possible) a path for a query, or else defaults to discrete verification or SMC.

## Philosophy

The core of Sally is a Model trait, which allows to model-check almost any untimed, timed, stochastic... semantics. Implementing this trait allows to define how a model should behave, as well as describing it's characteristics and giving some metadata about the semantics.
For instance, each model should give an implementation for the function _next_, which takes a state and an action, and returns the next state (or None if there are none).

Every model in Sally works with the same state structure : ModelState. This structure contains :
- Discrete variables, which are used in query evaluation, bounding... The number of discrete variables is fixed when building the model.
- Clocks, which can in fact be any continuous variables, and are used for query evaluation as well. The number of clocks is fixed when building the model.
- Deadlock flag,
- A vector of ModelStorage variables, a ModelStorage being a recursive enum allowing to represent almost any data structure. It is not used for query evaluation and any model is free to use it as it wants, allowing unbounded data-structures and models (stack-machines...). The number of storages is fixed when building the model, although each storage isn't necessarily bounded.

To define the anatomy of states for a network of Models, each Model must be compiled against a global ModelContext.
The compilation step is used to register models, variables, clocks, actions, and model storages. Variables, clocks, and actions must be named, and thus a ModelContext can also have scopes to allow putting several instances of the same model in a network without sharing the variables. 
The context then describe the current network, everything that is contained in a state, the addresses of variables and clocks... This is useful to create automatic networks of structures implementing Model, even if they are not from the same semantics.

In the event of model-checking a single model, the Model trait offers a _singleton_ method to automatically create a context using the _compile_ function.