use sas_math_core::config::GameConfig;
use sas_math_core::mechanics::Prepared;

// cap_x just under u64::MAX boundary: c < u64::MAX as f64 is true for c == (u64::MAX as f64) - small,
// but u64::MAX as f64 rounds UP to 2^64, so c.floor() as u64 can overflow-saturate. Check.
#[test]
fn huge_cap_x() {
    // bet=1, cap_x=1.8e19 (~ near u64::MAX=1.844e19). c = 1.8e19, finite, < 1.844e19? yes.
    // floor as u64 fine. Try cap_x=1.84e19:
    for cx in ["1.0e19", "1.8446744073709552e19", "1.9e19"] {
        let j = format!(r#"{{
          "schema_version":"0.1.0",
          "game":{{"name":"x","grid":{{"reels":2,"rows":1}},"bet_units":1}},
          "symbols":[{{"id":0,"name":"A","kind":"regular"}}],
          "reels":{{"strips":[[0],[0]]}},
          "paytable":[{{"symbol_id":0,"pays":{{"2":5}}}}],
          "paylines":[[0,0]],
          "line_eval":"left_to_right",
          "max_win":{{"cap_x":{cx}}},
          "rng":{{"generator":"chacha","master_seed":1}}
        }}"#);
        let cfg = GameConfig::from_json_str(&j).unwrap();
        let prep = Prepared::new(&cfg).unwrap();
        println!("cap_x={cx} -> cap_units={}", prep.cap_units);
    }
}

// Duplicate symbol ids in symbols array: last write wins for kind. And n_symbols uses MAX id.
// A gap in ids (id 0 and id 5, no 1-4) -> n_symbols=6, indices 1-4 are zero/regular phantom symbols.
#[test]
fn sparse_symbol_ids() {
    let j = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"x","grid":{"reels":2,"rows":1},"bet_units":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":5,"name":"B","kind":"scatter"}],
      "reels":{"strips":[[0,5],[0,5]]},
      "paytable":[{"symbol_id":5,"pays":{"2":5}}],
      "paylines":[[0,0]],
      "line_eval":"left_to_right",
      "max_win":{"cap_x":1000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let cfg = GameConfig::from_json_str(j).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    println!("sparse ids: n_symbols={} (phantom regulars 1-4 exist), scatter_id={:?}", prep.n_symbols, prep.scatter_id);
}

// duplicate scatter symbols: two scatters -> scatter_id keeps only the LAST. The other scatter's
// is_scatter=true but scatter_pay only reads ONE id.
#[test]
fn two_scatters() {
    let j = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"x","grid":{"reels":2,"rows":1},"bet_units":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"S1","kind":"scatter"},{"id":2,"name":"S2","kind":"scatter"}],
      "reels":{"strips":[[0,1,2],[0,1,2]]},
      "paytable":[{"symbol_id":1,"pays":{"2":5}},{"symbol_id":2,"pays":{"2":99}}],
      "paylines":[[0,0]],
      "line_eval":"left_to_right",
      "max_win":{"cap_x":1000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let cfg = GameConfig::from_json_str(j).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    println!("two scatters: scatter_id={:?} (only one wins; both count toward sc). is_scatter={:?}", prep.scatter_id, prep.is_scatter);
}
