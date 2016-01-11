rust-sat
========

A SAT solver that accepts input in the DIMACS CNF file format. There are three different types of solvers. One that uses naive, chronological backtracking, one that uses watch lists and a final one that does non-chronological backtracking.

```
Usage: rust-sat [--solver TYPE] <inputfile>
       rust-sat --help

Options:
    --solver TYPE  Valid values: naive, watch, nonchro.
    --help         Show this message.
```
