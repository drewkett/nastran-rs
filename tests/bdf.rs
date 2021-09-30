//extern crate nastran;
//use nastran::bdf::parser::parse_bytes_iter;
//
//const DATFILE: &'static [u8] = b"\
//PARAM,POST , 1 $ABC
//PARAM, WTMASS,0.00259
//+,1,2
//ABCDEF,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456,123456
//BLAH    123      1.+5   1e2     ABC
//GRID*            1100001               0    3.732130e+02    3.329000e+00 ED00013
//*ED00013    7.408100e+01               0
//                                          0.      0.059  0.      0.059 		1
//";
//
////      --------        --------        --------        --------        --------
//const CLEAN_DATFILE: &'static str = "\
//PARAM   POST    1                                                               $ABC
//PARAM   WTMASS  2.590e-3                                                +       
//+       1       2                                                               
//ABCDEF  123456  123456  123456  123456  123456  123456  123456  123456  +       
//+       123456  123456  123                                                     
//BLAH    123     100000.0100.0   ABC                                             
//GRID    1100001 0       3.7321e23.3290e07.4081e10                       +       
//+                                       0.0     5.900e-20.0     5.900e-2        
//";
//
//#[test]
//fn comma_separated() {
//    use std::fmt::Write;
//    let mut it = parse_bytes_iter(DATFILE.into_iter().cloned().map(Ok));
//    let mut out = String::new();
//    while let Some(card) = it.next() {
//        write!(out, "{}", card.unwrap()).unwrap();
//    }
//    for (line1, line2) in out.split("\n").zip(CLEAN_DATFILE.split('\n')) {
//        assert_eq!(line1, line2);
//    }
//}
