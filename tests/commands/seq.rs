use crate::command_test;

command_test!("seq", {
    pos_inc: [ cmd "1..10" assert "" => "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n" ],
    pos_dec: [ cmd "10..1" assert "" => "10\n9\n8\n7\n6\n5\n4\n3\n2\n1\n" ],
    neg_inc: [ cmd "-10..-1" assert "" => "-10\n-9\n-8\n-7\n-6\n-5\n-4\n-3\n-2\n-1\n" ],
    neg_dec: [ cmd "-1..-10" assert "" => "-1\n-2\n-3\n-4\n-5\n-6\n-7\n-8\n-9\n-10\n" ],
    sign_inc: [ cmd "-1..1" assert "" => "-1\n0\n1\n" ],
    sign_dec: [ cmd "1..-1" assert "" => "1\n0\n-1\n" ],
    pos_step: [ cmd "2..9" "2" assert "" => "2\n4\n6\n8\n" ],
    pos_step_2: [ cmd "2..10" "2" assert "" => "2\n4\n6\n8\n10\n" ],
    neg_step: [ cmd "9..2" "-3" assert "" => "9\n6\n3\n" ],
    neg_step_2: [ cmd "9..0" "-3" assert "" => "9\n6\n3\n0\n" ],
    single_neg: [ cmd "-1..-1" assert "" => "-1\n" ],
    single_zero: [ cmd "0..0" assert "" => "0\n" ],
    single_pos: [ cmd "1..1" assert "" => "1\n" ],
    inf: [ sh "%cmd% | head -n10" assert "" => "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n" ],
    inf_from: [ sh "%cmd% 10.. | head -n10" assert "" => "10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n" ],
    inf_pos_step: [ sh "%cmd% 10.. 2 | head -n10" assert "" => "10\n12\n14\n16\n18\n20\n22\n24\n26\n28\n" ],
    inf_neg_step: [ sh "%cmd% 10.. -3 | head -n10" assert "" => "10\n7\n4\n1\n-2\n-5\n-8\n-11\n-14\n-17\n" ],
    overflow: [ cmd "170141183460469231731687303715884105727.." assert "" => err "number sequence reached integer limit\n" ],
});
