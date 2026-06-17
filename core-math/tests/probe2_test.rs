// Probe 2: same-SYMBOL partial overlap (true double count) with both symbols paying + wilds.
use sas_math_core::config::GameConfig;
use sas_math_core::mechanics::Prepared;
fn prep(json:&str)->Prepared{Prepared::new(&GameConfig::from_json_str(json).unwrap()).unwrap()}
#[test]
fn same_symbol_overlap(){
    // A,B both pay; W wild. Look for a line where LTR and RTL resolve to the SAME symbol, runs overlap, not full.
    let _p = prep(r#"{
      "schema_version":"0.1.0",
      "game":{"name":"bw","grid":{"reels":5,"rows":1},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"},{"id":2,"name":"W","kind":"wild"}],
      "reels":{"strips":[[0,1,2],[0,1,2],[0,1,2],[0,1,2],[0,1,2]]},
      "paytable":[{"symbol_id":0,"pays":{"2":10,"3":30,"4":60,"5":100}},{"symbol_id":1,"pays":{"2":10,"3":30,"4":60,"5":100}}],
      "paylines":[[0,0,0,0,0]],
      "line_eval":"both_ways","max_win":{"cap_x":1000000},"rng":{"generator":"chacha","master_seed":1}
    }"#);
    let n=5usize;
    let is_wild=|s:usize| s==2;
    let pays=|sym:usize,c:usize|->u64{ if c>=2&&c<=5 {match c{2=>10,3=>30,4=>60,5=>100,_=>0}} else {0} };
    let run_len=|line:&[usize],sym:usize|->usize{ let sw=is_wild(sym); let mut r=0; for &s in line{ let m= if sw {s==sym} else {s==sym||is_wild(s)}; if m{r+=1}else{break} } r };
    let ltr=|line:&[usize]|->(usize,usize,u64){
        if is_wild(line[0]) { let mut b=(0,0,0u64); for sym in 0..2 { let c=run_len(line,sym); let p=pays(sym,c); if p>b.2{b=(sym,c,p);} } b }
        else { let s=line[0]; let c=run_len(line,s); (s,c,pays(s,c)) }
    };
    let mut found=0;
    for code in 0..3u32.pow(5){ let mut line=[0;5]; let mut c=code; for i in 0..n{line[i]=(c%3)as usize;c/=3;}
        let (ls,lr,lp)=ltr(&line); let mut rev=[0;5]; for i in 0..n{rev[i]=line[n-1-i];} let (rs,rr,rp)=ltr(&rev);
        if lp>0&&rp>0&&ls==rs&&lr!=n&&(lr+rr)>n { eprintln!("SAME-SYM OVERLAP {:?} sym{} L{} R{}",line,ls,lr,rr); found+=1; }
    }
    eprintln!("same-symbol non-full overlaps: {}", found);
}
