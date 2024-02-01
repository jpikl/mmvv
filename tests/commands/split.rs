use crate::command_test;

command_test!("split", {
    empty_0: [ cmd ":" should "" => "\n" ],
    empty_0_nl: [ cmd ":" should "\n" => "\n" ],
    empty_1: [ cmd ":" should ":" => "\n\n" ],
    empty_1_nl: [ cmd ":" should ":\n" => "\n\n" ],
    empty_2: [ cmd ":" should "::" => "\n\n\n" ],
    empty_2_nl: [ cmd ":" should "::\n" => "\n\n\n" ],
    many: [ cmd ":" should "a:bc:def" => "a\nbc\ndef\n" ],
    many_nl: [ cmd ":" should "a:bc:def\n" => "a\nbc\ndef\n" ],
    many_trail: [ cmd ":" should "a:bc:def:" => "a\nbc\ndef\n\n" ],
    many_trail_nl: [ cmd ":" should "a:bc:def:\n" => "a\nbc\ndef\n\n" ],
    buf_under: [ cmd ":" "--buf-size=8" should "aaaaaa:\n" => "aaaaaa\n\n" ],
    buf_exact: [ cmd ":" "--buf-size=8" should "aaaaaaa:\n" => "aaaaaaa\n\n" ],
    buf_over: [ cmd ":" "--buf-size=8" should "aaaaaaaa:\n" => "aaaaaaaa\n\n" ],
    buf_over_2: [ cmd ":" "--buf-size=8" should "aaaaaaaaa:\n" => "aaaaaaaaa\n\n" ],
    trail_empty_0: [ cmd "-t" ":" should "" => "\n" ],
    trail_empty_0_nl: [ cmd "-t" ":" should "\n" => "\n" ],
    trail_empty_1: [ cmd "-t" ":" should ":" => "\n" ],
    trail_empty_1_nl: [ cmd "-t" ":" should ":\n" => "\n" ],
    trail_empty_2: [ cmd "-t" ":" should "::" => "\n\n" ],
    trail_empty_2_nl: [ cmd "-t" ":" should "::\n" => "\n\n" ],
    trail_many: [ cmd "-t" ":" should "a:bc:def" => "a\nbc\ndef\n" ],
    trail_many_nl: [ cmd "-t" ":" should "a:bc:def\n" => "a\nbc\ndef\n" ],
    trail_many_trail: [ cmd "-t" ":" should "a:bc:def:" => "a\nbc\ndef\n" ],
    trail_many_trail_nl: [ cmd "-t" ":" should "a:bc:def:\n" => "a\nbc\ndef\n" ],
    trail_buf_under: [ cmd "-t" ":" "--buf-size=8" should "aaaaaa:\n" => "aaaaaa\n" ],
    // FIXME

    // trail_buf_exact: [ cmd "-t" ":" "--buf-size=8" should "aaaaaaa:\n" => "aaaaaaa\n" ],
    trail_buf_over: [ cmd "-t" ":" "--buf-size=8" should "aaaaaaaa:\n" => "aaaaaaaa\n" ],
    trail_buf_over_2: [ cmd "-t" ":" "--buf-size=8" should "aaaaaaaaa:\n" => "aaaaaaaaa\n" ],
});
