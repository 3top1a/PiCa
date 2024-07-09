# PiCA

PiCA (*rPi Chess Automaton*, or *Practically Imbecilic Checkmate Attempter*) is my fourth attempt at creating a competent chess engine.
It is written in Rust and optimized for the hardware of a Raspberry Pi Zero 2 W.

Current features:
- Alpha beta/Negamax search
- Piece square tables
- Check Extensions
- Quiescence search
- Iterative deepening
- Transposition table
- Delta Pruning
- Move ordering
  - MVV-LVA
  - Hash move
  - Killers
  - History Heuristic
  - PV


## Resources

A list of resources that have helped me
- CPW obviously
- https://web.archive.org/web/20070607231311/http://www.brucemo.com/compchess/programming/index.htm
- [Vice tutorial series](https://www.youtube.com/watch?v=bGAfaepBco4&list=PLZ1QII7yudbc-Ky058TEaOstZHVbT-2hg)


| version | elo change     | LOS  |
|---------|----------------|------|
| 0.0.3   | ?              | ?    |
| 0.0.4   | 249.1 +/- 41.2 | 100% |
| 0.0.5   | 96.2 +/- 47.8  | 100% |
