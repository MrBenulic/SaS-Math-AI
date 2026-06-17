use sas_math_core::config::GameConfig;
use sas_math_core::mechanics::Prepared;

// 1. Empty symbols array: n_symbols=0 -> max_count=0 -> pays vec empty. Does Prepared accept it?
#[test]
fn empty_symbols_accepted() {
    let j = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"x","grid":{"reels":2,"rows":1},"bet_units":1},
      "symbols":[],
      "reels":{"strips":[[0],[0]]},
      "paytable":[],
      "paylines":[[0,0]],
      "line_eval":"left_to_right",
      "max_win":{"cap_x":10},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let cfg = GameConfig::from_json_str(j).unwrap();
    let r = Prepared::new(&cfg);
    println!("empty symbols + strip referencing sym 0: {:?}", r.as_ref().err());
}

// 2. Strip referencing symbol id beyond symbols but within... actually covered. 
// 3. paytable count key "0" -> pays[sym][0] set. scatter_pay(0) would then pay on zero scatters!
#[test]
fn scatter_pay_on_zero_count() {
    let j = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"x","grid":{"reels":3,"rows":1},"bet_units":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"S","kind":"scatter"}],
      "reels":{"strips":[[0,1],[0,1],[0,1]]},
      "paytable":[{"symbol_id":1,"pays":{"0":7,"2":5}}],
      "paylines":[[0,0,0]],
      "line_eval":"left_to_right",
      "max_win":{"cap_x":1000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let cfg = GameConfig::from_json_str(j).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    println!("scatter_pay(0) = {} (pays[1][0] was set to 7!)", prep.scatter_pay(0));
    assert_eq!(prep.scatter_pay(0), 7, "BUG: a pays key of \"0\" pays out on ZERO scatters every spin");
}

// 4. Negative cap_x -> floor as u64 wraps?
#[test]
fn negative_cap_x() {
    let j = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"x","grid":{"reels":2,"rows":1},"bet_units":10},
      "symbols":[{"id":0,"name":"A","kind":"regular"}],
      "reels":{"strips":[[0],[0]]},
      "paytable":[{"symbol_id":0,"pays":{"2":5}}],
      "paylines":[[0,0]],
      "line_eval":"left_to_right",
      "max_win":{"cap_x":-5.0},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let cfg = GameConfig::from_json_str(j).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    println!("negative cap_x=-5 bet=10 -> cap_units = {}", prep.cap_units);
}

// 5. NaN cap_x
#[test]
fn nan_cap_x() {
    // JSON can't carry NaN, but a programmatic GameConfig could. Skip - JSON path can't reach it.
}
