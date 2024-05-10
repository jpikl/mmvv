use anyhow::format_err;
use anyhow::Result;
use clap::builder::PossibleValue;
use clap::builder::Str;
use clap::builder::StyledStr;
use clap::builder::ValueRange;
use clap::Arg;
use clap::Command;
use rew::command::Group;
use rew::command::Meta;
use rew::commands;
use rew::examples::Example;
use std::borrow::Cow;
use std::iter::Peekable;

pub struct Adapter<'a> {
    inner: &'a Command,
    parents: Vec<&'a Command>,
}

impl<'a> Adapter<'a> {
    pub fn new(inner: &'a Command) -> Self {
        Self {
            inner,
            parents: Vec::new(),
        }
    }

    fn parents_and_self(&self) -> Vec<&'a Command> {
        [&self.parents[..], &[self.inner]].concat()
    }

    fn meta(&self) -> Option<&'static Meta> {
        if self.parents.len() == 1 {
            commands::get_meta(self.name()) // Only the main commands have metadata
        } else {
            None
        }
    }

    fn group(&self) -> Group {
        self.meta().map(|sc| sc.group).unwrap_or_default()
    }

    pub fn examples(&self) -> impl Iterator<Item = &Example> {
        self.meta()
            .map(|meta| meta.examples)
            .unwrap_or_default()
            .iter()
    }

    pub fn name(&self) -> &str {
        self.inner.get_name()
    }

    pub fn full_name(&self) -> &str {
        self.inner
            .get_bin_name()
            .unwrap_or_else(|| self.inner.get_name())
    }

    pub fn file_stem(&self) -> &str {
        self.inner
            .get_display_name()
            .unwrap_or_else(|| self.inner.get_name())
    }

    pub fn description(&self) -> Result<&StyledStr> {
        self.inner
            .get_long_about()
            .map_or_else(|| self.short_description(), Ok)
    }

    pub fn short_description(&self) -> Result<&StyledStr> {
        self.inner
            .get_about()
            .ok_or_else(|| format_err!("command '{}' does not have description", self.full_name()))
    }

    pub fn synopsis_args(&'a self) -> Vec<SynopsisArg<'a>> {
        let mut args = Vec::new();

        if self.opt_args().next().is_some() {
            args.push(SynopsisArg::Options);
        }

        for arg in self.pos_args() {
            args.push(SynopsisArg::Positional(arg));
        }

        if self.subcommands().next().is_some() || self.name() == "help" {
            args.push(SynopsisArg::Commands(self));
        }

        args
    }

    pub fn subcommands(&'a self) -> impl Iterator<Item = Adapter<'a>> {
        self.inner
            .get_subcommands()
            .take_while(|_| self.name() != "help")
            .filter(|subcommand| !subcommand.is_hide_set())
            .map(|subcommand| Adapter {
                inner: subcommand,
                parents: self.parents_and_self(),
            })
    }

    pub fn groupped_subcommands(
        &'a self,
    ) -> impl Iterator<Item = (Group, impl Iterator<Item = Adapter<'a>>)> {
        Group::values().into_iter().filter_map(|group| {
            self.subcommands()
                .filter(move |subcommand| subcommand.group() == group)
                .non_empty()
                .map(|iter| (group, iter))
        })
    }

    pub fn pos_args(&self) -> impl Iterator<Item = PositionalArg<'_>> {
        self.inner
            .get_arguments()
            .filter(|arg| !arg.is_hide_set() && arg.is_positional())
            .map(BaseArg)
            .map(PositionalArg)
    }

    pub fn opt_args(&self) -> impl Iterator<Item = OptionalArg<'_>> {
        self.inner
            .get_arguments()
            .filter(|arg| !arg.is_hide_set() && !arg.is_positional() && !arg.is_global_set())
            .map(BaseArg)
            .map(OptionalArg)
    }

    pub fn global_opt_args(&self) -> impl Iterator<Item = OptionalArg<'_>> {
        self.inner
            .get_arguments()
            .filter(|arg| !arg.is_hide_set() && !arg.is_positional() && arg.is_global_set())
            .map(BaseArg)
            .map(OptionalArg)
    }

    pub fn own_args<'b>(
        &self,
        args: &'b [OptionalArg<'a>],
    ) -> impl Iterator<Item = &'b OptionalArg<'a>> {
        let parent_arg_ids = self
            .parents
            .iter()
            .flat_map(|parent| parent.get_arguments())
            .map(Arg::get_id)
            .collect::<Vec<_>>();

        args.iter()
            .filter(move |arg| !parent_arg_ids.contains(&arg.base().0.get_id()))
    }

    pub fn parent_with_args(&self, args: &[OptionalArg<'a>]) -> Option<Adapter<'a>> {
        let arg_ids = args
            .iter()
            .map(|arg| arg.base().0.get_id())
            .collect::<Vec<_>>();

        self.parents
            .iter()
            .rev()
            .find(|parent| {
                parent
                    .get_arguments()
                    .any(|arg| arg_ids.contains(&arg.get_id()))
            })
            .map(|parent| Adapter::new(parent))
    }
}
pub enum SynopsisArg<'a> {
    Options,
    Positional(PositionalArg<'a>),
    Commands(&'a Adapter<'a>),
}

impl<'a> SynopsisArg<'a> {
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            Self::Options => "OPTIONS".into(),
            Self::Positional(arg) => arg.name().into(),
            Self::Commands(command) => command
                .inner
                .get_subcommand_value_name()
                .unwrap_or("COMMAND")
                .into(),
        }
    }

    pub fn is_required(&self) -> bool {
        match self {
            Self::Options => false,
            Self::Positional(arg) => arg.is_required(),
            Self::Commands(command) => command.inner.is_subcommand_required_set(),
        }
    }

    pub fn is_many(&self) -> bool {
        match self {
            Self::Options => false,
            Self::Positional(arg) => arg.is_many(),
            Self::Commands(command) => command.name() == "help",
        }
    }
}

pub struct PositionalArg<'a>(BaseArg<'a>);

impl<'a> PositionalArg<'a> {
    pub fn base(&self) -> &BaseArg<'a> {
        &self.0
    }

    pub fn name(&self) -> String {
        self.base()
            .0
            .get_value_names()
            .unwrap_or_default()
            .iter()
            .next()
            .map_or_else(
                || self.base().0.get_id().to_string().to_uppercase(),
                ToString::to_string,
            )
    }

    pub fn is_required(&self) -> bool {
        self.base().0.is_required_set()
    }

    pub fn is_many(&self) -> bool {
        self.base().value_range().max_values() > 1
    }
}

pub struct OptionalArg<'a>(BaseArg<'a>);

impl<'a> OptionalArg<'a> {
    pub fn base(&self) -> &BaseArg<'a> {
        &self.0
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();

        if !self.base().0.is_hide_short_help_set() {
            if let Some(shorts) = self.base().0.get_short_and_visible_aliases() {
                for short in shorts {
                    names.push(format!("-{short}"));
                }
            }
        }

        if !self.base().0.is_hide_long_help_set() {
            if let Some(longs) = self.base().0.get_long_and_visible_aliases() {
                for long in longs {
                    names.push(format!("--{long}"));
                }
            }
        }

        names
    }

    pub fn value_names(&self) -> impl Iterator<Item = &Str> {
        self.base()
            .0
            .get_value_names()
            .unwrap_or_default()
            .iter()
            .take_while(move |_| self.base().value_range().takes_values())
    }
}

pub struct BaseArg<'a>(&'a Arg);

impl<'a> BaseArg<'a> {
    pub fn description(&self) -> Result<&StyledStr> {
        let long_help = if self.0.is_hide_long_help_set() {
            None
        } else {
            self.0.get_long_help()
        };

        let short_help = if self.0.is_hide_short_help_set() {
            None
        } else {
            self.0.get_help()
        };

        long_help
            .or(short_help)
            .ok_or_else(|| format_err!("argument '{}' does not have description", self.0.get_id()))
    }

    pub fn possible_values(&self) -> impl Iterator<Item = Value> {
        let is_hidden = self.0.is_hide_possible_values_set();
        let is_needed = self.0.is_positional() || self.value_range().takes_values();

        self.0
            .get_possible_values()
            .into_iter()
            .take_while(move |_| !is_hidden && is_needed)
            .filter(|value| !value.is_hide_set())
            .map(Value)
    }

    pub fn default_value(&self) -> Option<String> {
        let is_hidden = self.0.is_hide_default_value_set();
        let is_needed = self.value_range().takes_values();

        self.0
            .get_default_values()
            .iter()
            .take_while(move |_| !is_hidden && is_needed)
            .map(|value| value.to_string_lossy())
            .non_empty()
            .map(|iter| iter.collect::<Vec<_>>().join(","))
    }

    pub fn env_var(&self) -> Option<Cow<'_, str>> {
        if self.0.is_hide_env_set() {
            None
        } else {
            self.0.get_env().map(|value| value.to_string_lossy())
        }
    }

    fn value_range(&self) -> ValueRange {
        self.0
            .get_num_args()
            .expect("command.build() was not called")
    }
}

pub struct Value(PossibleValue);

impl Value {
    pub fn name(&self) -> &str {
        self.0.get_name()
    }

    pub fn description(&self) -> Result<&StyledStr> {
        self.0.get_help().ok_or_else(|| {
            format_err!(
                "Possible value '{}' does not have description",
                self.0.get_name()
            )
        })
    }
}

pub trait NonEmpty {
    fn non_empty(self) -> Option<impl Iterator>;
}

impl<I: Iterator> NonEmpty for I {
    fn non_empty(self) -> Option<Peekable<I>> {
        let mut iter = self.peekable();
        if iter.peek().is_some() {
            Some(iter)
        } else {
            None
        }
    }
}
