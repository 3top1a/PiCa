mod tests {
    use crate::*;

    macro_rules! nextmoveassert {
        ($fen:expr, $move:expr) => {
            use chess::ChessMove;
            let board = Board::from_str($fen).unwrap();
            let mv = Engine::new(256).start(board, TimeManager::test_preset());
            let bestmv = ChessMove::from_san(&board, $move).unwrap();
            assert_eq!(
                mv.to_string(),
                bestmv.to_string(),
                "{}",
                format!("FEN: {}", board)
            );
        };
    }

    /// Parses an EPD string and returns the FEN and the best move as a tuple
    fn parse_epd(epd: &str) -> (&str, &str) {
        let parts: Vec<&str> = epd.split(" bm ").collect();
        let fen = parts[0];
        let best_move = parts[1].split(';').next().unwrap().trim();
        (fen, best_move)
    }

    // Test position tests
    // https://groups.google.com/g/rec.games.chess.misc/c/OeK0k5KDaf4

    /// Bratko Kopec Test
    /// https://www.chessprogramming.org/Bratko-Kopec_Test
    #[test]
    fn bratko_kopec() {
        let positions = "1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - - bm Qd1+;
3r1k2/4npp1/1ppr3p/p6P/P2PPPP1/1NR5/5K2/2R5 w - - bm d5;
2q1rr1k/3bbnnp/p2p1pp1/2pPp3/PpP1P1P1/1P2BNNP/2BQ1PRK/7R b - - bm f5;
rnbqkb1r/p3pppp/1p6/2ppP3/3N4/2P5/PPP1QPPP/R1B1KB1R w KQkq - bm e6;
r1b2rk1/2q1b1pp/p2ppn2/1p6/3QP3/1BN1B3/PPP3PP/R4RK1 w - - bm Nd5 a4;
2r3k1/pppR1pp1/4p3/4P1P1/5P2/1P4K1/P1P5/8 w - - bm g6;
1nk1r1r1/pp2n1pp/4p3/q2pPp1N/b1pP1P2/B1P2R2/2P1B1PP/R2Q2K1 w - - bm Nf6;
4b3/p3kp2/6p1/3pP2p/2pP1P2/4K1P1/P3N2P/8 w - - bm f5;
2kr1bnr/pbpq4/2n1pp2/3p3p/3P1P1B/2N2N1Q/PPP3PP/2KR1B1R w - - bm f5;
3rr1k1/pp3pp1/1qn2np1/8/3p4/PP1R1P2/2P1NQPP/R1B3K1 b - - bm Ne5;
2r1nrk1/p2q1ppp/bp1p4/n1pPp3/P1P1P3/2PBB1N1/4QPPP/R4RK1 w - - bm f4;
r3r1k1/ppqb1ppp/8/4p1NQ/8/2P5/PP3PPP/R3R1K1 b - - bm Bf5;
r2q1rk1/4bppp/p2p4/2pP4/3pP3/3Q4/PP1B1PPP/R3R1K1 w - - bm b4;
rnb2r1k/pp2p2p/2pp2p1/q2P1p2/8/1Pb2NP1/PB2PPBP/R2Q1RK1 w - - bm Qd2 Qe1;
2r3k1/1p2q1pp/2b1pr2/p1pp4/6Q1/1P1PP1R1/P1PN2PP/5RK1 w - - bm Qxg7+;
r1bqkb1r/4npp1/p1p4p/1p1pP1B1/8/1B6/PPPN1PPP/R2Q1RK1 w kq - bm Ne4;
r2q1rk1/1ppnbppp/p2p1nb1/3Pp3/2P1P1P1/2N2N1P/PPB1QP2/R1B2RK1 b - - bm h5;
r1bq1rk1/pp2ppbp/2np2p1/2n5/P3PP2/N1P2N2/1PB3PP/R1B1QRK1 b - - bm Nb3;
3rr3/2pq2pk/p2p1pnp/8/2QBPP2/1P6/P5PP/4RRK1 b - - bm Rxe4;
r4k2/pb2bp1r/1p1qp2p/3pNp2/3P1P2/2N3P1/PPP1Q2P/2KRR3 w - - bm g4;
3rn2k/ppb2rpp/2ppqp2/5N2/2P1P3/1P5Q/PB3PPP/3RR1K1 w - - bm Nh6;
2r2rk1/1bqnbpp1/1p1ppn1p/pP6/N1P1P3/P2B1N1P/1B2QPP1/R2R2K1 b - - bm Bxe4;
r1bqk2r/pp2bppp/2p5/3pP3/P2Q1P2/2N1B3/1PP3PP/R4RK1 b kq - bm f6;
r2qnrnk/p2b2b1/1p1p2pp/2pPpp2/1PP1P3/PRNBB3/3QNPPP/5RK1 w - - bm f4;"
            .lines();

        for pos in positions {
            let (fen, best_move) = parse_epd(pos);
            nextmoveassert!(fen, best_move);
        }
    }

    #[test]
    /// https://www.stmintz.com/ccc/index.php?id=476109
    fn endgames() {
        let engine = Engine::new(256);

        // https://www.stmintz.com/ccc/index.php?id=391553
        let positions = "3k4/8/4K3/2R5/8/8/8/8 w - - bm Rc1
4k3/8/4K3/8/8/8/2R5/8 w - - 2 2 bm Rc8
1k6/7R/2K5/8/8/8/8/8 w - - bm Rh8
8/3k4/8/8/3PK3/8/8/8 w - - bm Kd5
2k5/8/1K1P4/8/8/8/8/8 w - - bm Kc6"
            .lines();

        for pos in positions {
            let (fen, best_move) = parse_epd(pos);
            nextmoveassert!(fen, best_move);
        }
    }
}
