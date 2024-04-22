# Modular Rust Model-Checker (WIP)

Sally will be a modular model checker, able to handle multiple model semantics, apply translation between semantics, compute analytic solutions as well as performing statistical model checking and discrete verification.

It's primary objective is the verification of Time Petri nets, but new semantics can easily be added by implementing traits.

The verification engine will be based on a "Model solving graph", where vertices are semantics, edges are translations between semantics, and analytic solutions are annoted to the corresponding semantic.
