#!/bin/bash

mkdir testout
id=$(date +%H:%M-%d.%m)

cutechess-cli \
-engine conf="pica" \
-engine conf="pica-old" \
-each \
    tc=inf/10+0.1 \
-games 2 -rounds 2500 -repeat 2 -maxmoves 200 \
-openings file=silversuite.pgn plies=20 \
-sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 \
-concurrency 8 \
-ratinginterval 10 \
-recover \
-pgnout "testout/sprt-${id}.pgn"
