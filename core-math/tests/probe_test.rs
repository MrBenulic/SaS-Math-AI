// Probe: brute-force search for both_ways partial-overlap double counts and ltr_run edge cases.
use sas_math_core::config::GameConfig;
use sas_math_core::mechanics::Prepared;

fn prep(json: &str) -> Prepared {
    Prepared::new(&GameConfig::from_json_str(json).unwrap()).unwrap()
}

// 5x1 both_ways: symbols A(0) regular, W(2) wild, B(1) regular. A pays for 2..5.
fn prep_bw_wild() -> Prepared {
    let json = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"bw","grid":{"reels":5,"rows":1},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"},{"id":2,"name":"W","kind":"wild"}],
      "reels":{"strips":[[0,1,2],[0,1,2],[0,1,2],[0,1,2],[0,1,2]]},
      "paytable":[{"symbol_id":0,"pays":{"2":10,"3":30,"4":60,"5":100}}],
      "paylines":[[0,0,0,0,0]],
      "line_eval":"both_ways",
      "max_win":{"cap_x":1000000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    prep(json)
}

// "true" both_ways with explicit cell-set union dedup, treating each direction's run as a SET of cells,
// paying the higher-value of overlapping wins (the standard "no cell paid twice" semantics is NOT what
// the engine does, but let's first detect ANY overlap where engine pays both).
#[test]
fn search_partial_overlap() {
    let p = prep_bw_wild();
    let n = 5;
    let mut found = Vec::new();
    // enumerate all lines over {0,1,2}^5
    for code in 0..3u32.pow(5) {
        let mut line = [0usize; 5];
        let mut c = code;
        for i in 0..n { line[i] = (c % 3) as usize; c /= 3; }

        // replicate engine ltr_run for both directions to get run lengths
        let (lsym, lrun, lpay) = ltr(&p, &line);
        let mut rev = [0usize;5];
        for i in 0..n { rev[i] = line[n-1-i]; }
        let (rsym, rrun, rpay) = ltr(&p, &rev);

        // engine pays: if lrun==n -> lpay once; else lpay+rpay
        // overlap exists if both pay and lrun+rrun > n (share >=1 cell) and lrun != n
        if lpay>0 && rpay>0 && lrun != n && (lrun + rrun) > n {
            found.push((line, lsym, lrun, lpay, rsym, rrun, rpay));
        }
    }
    for f in &found {
        eprintln!("OVERLAP line={:?} L(sym{},run{},pay{}) R(sym{},run{},pay{})", f.0, f.1,f.2,f.3,f.4,f.5,f.6);
    }
    eprintln!("total partial-overlap lines where engine pays BOTH: {}", found.len());
}

fn ltr(p: &Prepared, line: &[usize]) -> (usize, usize, u64) {
    // we cannot call private ltr_run; reconstruct via line_wins for LTR-only by using left_to_right model?
    // Instead use line_pay difference: not possible. Use a re-impl matching the source.
    let is_wild = |s: usize| s == 2;
    let is_scatter = |_s: usize| false;
    if line.is_empty() || is_scatter(line[0]) { return (0,0,0); }
    let max_count = 5usize;
    let pays = |sym: usize, c: usize| -> u64 {
        if c>=1 && c<=max_count {
            // A(0): 2->10,3->30,4->60,5->100
            if sym==0 { match c {2=>10,3=>30,4=>60,5=>100,_=>0} } else {0}
        } else {0}
    };
    let run_len = |sym: usize| -> usize {
        let sym_is_wild = is_wild(sym);
        let mut run=0;
        for &s in line {
            let m = if sym_is_wild { s==sym } else { s==sym || is_wild(s) };
            if m { run+=1 } else { break }
        }
        run
    };
    if is_wild(line[0]) {
        let mut best=(0usize,0usize,0u64);
        for sym in 0..3 {
            if is_scatter(sym) {continue}
            let c=run_len(sym);
            let pp=pays(sym,c);
            if pp>best.2 { best=(sym,c,pp); }
        }
        best
    } else {
        let s=line[0];
        let c=run_len(s);
        (s,c,pays(s,c))
    }
}
