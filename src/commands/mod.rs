use crate::command::Meta;

mod ascii;
mod cat;
mod first;
mod join;
mod last;
mod r#loop;
mod lower;
mod seq;
mod skip;
mod split;
mod stream;
mod trim;
mod upper;
mod x;

pub const COMMANDS: &[&Meta] = &[
    &ascii::META,
    &cat::META,
    &first::META,
    &join::META,
    &last::META,
    &r#loop::META,
    &lower::META,
    &seq::META,
    &skip::META,
    &split::META,
    &stream::META,
    &trim::META,
    &upper::META,
    &x::META,
];

pub fn get_meta(name: &str) -> Option<&'static Meta> {
    COMMANDS.iter().find(|meta| meta.name == name).copied()
}
