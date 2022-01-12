// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/*! LLVM command option parsing.

This crate provides a mechanism for parsing arguments to LLVM programs.

It does so by consuming the LLVM tablegen data defining command options
and reimplementing an argument parser that takes this rich metadata into
account. This (hopefully) enables argument parsing to abide by the same
semantics. This includes the ability to recognize aliases and properly
recognize variations on argument parsing. e.g. `-I<value>` and `-I <value>`
being semantically equivalent.

# Higher-Level API

The API provided is currently rather low-level. We desire to implement a
high-level API someday. For example, we want to turn clang's parsed options
into structs that convey the meaning of each invocation, such as whether we're
invoking a compiler, linker, etc.
 */

use {
    serde::Deserialize,
    serde_json::Value,
    std::{
        borrow::Cow,
        collections::HashMap,
        ffi::{OsStr, OsString},
        str::FromStr,
    },
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("unrecognized argument prefix: {0}")]
    UnrecognizedArgumentPrefix(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("JSON parsing error: {0}")]
    JsonParse(String),

    #[error("argument {0} missing required value")]
    ParseNoArgumentValue(String),

    #[error("argument {0} expected {1} values but only got {2}")]
    ParseMultipleValuesMissing(String, usize, usize),

    #[error("failed to resolve option alias {0} to {1}")]
    AliasMissing(String, String),
}

#[cfg(target_family = "unix")]
use std::os::unix::ffi::OsStrExt;

#[cfg(target_family = "windows")]
use std::os::windows::ffi::{OsStrExt, OsStringExt};

#[cfg(target_family = "unix")]
fn argument_matches_exact(arg: &ProgramOption, s: &OsStr) -> bool {
    arg.prefixes
        .iter()
        .any(|prefix| s.as_bytes() == prefix.with_name(&arg.name).as_bytes())
}

#[cfg(target_family = "windows")]
fn argument_matches_exact(arg: &ProgramOption, s: &OsStr) -> bool {
    arg.prefixes
        .iter()
        .any(|prefix| s == OsString::from(prefix.with_name(&arg.name)))
}

#[cfg(target_family = "unix")]
fn argument_matches_prefix<'a>(arg: &ProgramOption, s: &'a OsStr) -> Option<Cow<'a, OsStr>> {
    let s_bytes = s.as_bytes();

    for prefix in &arg.prefixes {
        let search = prefix.with_name(&arg.name);

        if s_bytes.starts_with(search.as_bytes()) {
            return Some(Cow::Borrowed(OsStr::from_bytes(
                &s_bytes[search.as_bytes().len()..],
            )));
        }
    }

    None
}

#[cfg(target_family = "windows")]
fn argument_matches_prefix<'a>(arg: &ProgramOption, s: &'a OsStr) -> Option<Cow<'a, OsStr>> {
    for prefix in &arg.prefixes {
        let search = OsString::from(prefix.with_name(&arg.name));

        // There isn't an OsStr.starts_with(). So roll our own by comparing iterators.

        let search_chars = search.encode_wide().count();

        if search.encode_wide().eq(s.encode_wide().take(search_chars)) {
            let remaining =
                OsString::from_wide(&s.encode_wide().skip(search_chars).collect::<Vec<_>>());

            return Some(Cow::Owned(remaining));
        }
    }

    None
}

/// Maps to `llvm-tblgen` JSON maps defining a single program option.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JsonOption {
    alias: Option<JsonOptionAlias>,
    flags: Option<Vec<JsonOptionFlag>>,
    group: Option<JsonOptionGroup>,
    kind: Option<JsonOptionKind>,
    name: Option<String>,
    num_args: Option<usize>,
    prefixes: Option<Vec<String>>,
    #[serde(rename = "!name")]
    raw_name: String,
    #[serde(rename = "!superclasses")]
    super_classes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct JsonOptionAlias {
    def: String,
    #[allow(unused)]
    kind: String,
    #[allow(unused)]
    printable: String,
}

#[derive(Clone, Debug, Deserialize)]
struct JsonOptionFlag {
    def: String,
    #[allow(unused)]
    kind: String,
    #[allow(unused)]
    printable: String,
}

#[derive(Clone, Debug, Deserialize)]
struct JsonOptionGroup {
    def: String,
    #[allow(unused)]
    kind: String,
    #[allow(unused)]
    printable: String,
}

#[derive(Clone, Debug, Deserialize)]
struct JsonOptionKind {
    def: String,
    #[allow(unused)]
    kind: String,
    #[allow(unused)]
    printable: String,
}

/// The prefix for an argument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentPrefix {
    /// Prefixed by `-`.
    SingleDash,
    /// Prefixed by `--`.
    DoubleDash,
    /// Prefixed by `-?`.
    SingleDashQuestion,
    /// Prefixed by `/`.
    Slash,
    /// Prefixed by `/?`.
    SlashQuestion,
}

impl FromStr for ArgumentPrefix {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(Self::SingleDash),
            "--" => Ok(Self::DoubleDash),
            "-?" => Ok(Self::SingleDashQuestion),
            "/" => Ok(Self::Slash),
            "/?" => Ok(Self::SlashQuestion),
            _ => Err(Error::UnrecognizedArgumentPrefix(s.to_string())),
        }
    }
}

impl ToString for ArgumentPrefix {
    fn to_string(&self) -> String {
        match self {
            Self::SingleDash => "-",
            Self::DoubleDash => "--",
            Self::SingleDashQuestion => "-?",
            Self::Slash => "/",
            Self::SlashQuestion => "/?",
        }
        .to_string()
    }
}

impl ArgumentPrefix {
    /// Format the prefix with a given argument name after it.
    pub fn with_name(&self, name: &str) -> String {
        format!("{}{}", self.to_string(), name)
    }
}

/// The kind of an LLVM option.
///
/// These correspond to the KIND_* definitions in llvm/Option/OptParser.td.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionKind {
    /// The input option kind.
    Input,

    /// Unknown option kind.
    Unknown,

    /// A flag with no values.
    ///
    /// e.g. `-pthread`.
    Flag,

    /// An option which prefixes its single value.
    ///
    /// e.g. `-O<value>`.
    Joined,

    /// An option which is followed by its value.
    ///
    /// e.g. `-Xpreprocessor <value>`.
    Separate,

    /// An option followed by its values, which are separated by commas.
    ///
    /// e.g. `-fno_sanitize_coverage=foo,bar`.
    CommaJoined,

    /// An option which takes multiple separate arguments.
    ///
    /// e.g. `-fget-definition foo bar baz`.
    MultiArg(usize),

    /// An option which is either joined to its non-empty value or followed by its value.
    ///
    /// e.g. `I` can be expressed as either `-I<value>` or `-I <value>`.
    JoinedOrSeparate,

    /// An option which is both joined to its first value and followed by its second value.
    ///
    /// e.g. `-Xarch__<key> <value>`.
    JoinedAndSeparate,

    /// An option which consumes all remaining arguments if there are any.
    RemainingArgs,

    /// An option which consumes an optional joined argument and any other remaining arguments.
    RemainingArgsJoined,
}

impl OptionKind {
    /// Whether the syntax of this kind matches if an equal sign is present.
    pub fn syntax_matches_with_equals(&self) -> bool {
        matches!(self, Self::Flag | Self::Separate | Self::MultiArg(_))
    }
}

impl TryFrom<&JsonOption> for OptionKind {
    type Error = Error;

    fn try_from(value: &JsonOption) -> Result<Self, Self::Error> {
        if let Some(kind) = &value.kind {
            match kind.def.as_str() {
                "KIND_COMMAJOINED" => Ok(Self::CommaJoined),
                "KIND_INPUT" => Ok(Self::Input),
                "KIND_JOINED" => Ok(Self::Joined),
                "KIND_JOINED_AND_SEPARATE" => Ok(Self::JoinedAndSeparate),
                "KIND_JOINED_OR_SEPARATE" => Ok(Self::JoinedOrSeparate),
                "KIND_FLAG" => Ok(Self::Flag),
                "KIND_MULTIARG" => Ok(Self::MultiArg(value.num_args.ok_or_else(|| {
                    Error::JsonParse("NumArgs should be present when .Kind is present".into())
                })?)),
                "KIND_REMAINING_ARGS" => Ok(Self::RemainingArgs),
                "KIND_REMAINING_ARGS_JOINED" => Ok(Self::RemainingArgsJoined),
                "KIND_SEPARATE" => Ok(Self::Separate),
                "KIND_UNKNOWN" => Ok(Self::Unknown),
                value => Err(Error::JsonParse(format!(
                    "option kind {} not recognized (please report this bug)",
                    value
                ))),
            }
        } else {
            Err(Error::JsonParse(".Kind not present".into()))
        }
    }
}

/// An option passed to an LLVM based program.
///
/// This defines an abstract option that can be passed to a program. It
/// is derived from tblgen files in LLVM's source repository.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramOption {
    /// Internal name from TBD.
    pub option_name: String,
    /// Argument name.
    pub name: String,
    /// The kind/syntax of the option.
    pub kind: OptionKind,
    /// The prefixes for this argument.
    pub prefixes: Vec<ArgumentPrefix>,
    /// Name of option this is an alias for.
    pub alias: Option<String>,
    /// Flags associated with this option.
    pub flags: Vec<String>,
    /// The group this option is part of.
    pub group: Option<String>,
}

impl PartialOrd for ProgramOption {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a_matches_with_equals = self.kind.syntax_matches_with_equals();
        let b_matches_with_equals = other.kind.syntax_matches_with_equals();

        if a_matches_with_equals && !b_matches_with_equals {
            Some(std::cmp::Ordering::Less)
        } else if !a_matches_with_equals && b_matches_with_equals {
            Some(std::cmp::Ordering::Greater)
        } else if !a_matches_with_equals
            && !b_matches_with_equals
            && self.name.len() != other.name.len()
        {
            Some(other.name.len().cmp(&self.name.len()))
        } else {
            Some(self.option_name.cmp(&other.option_name))
        }
    }
}

impl Ord for ProgramOption {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl ProgramOption {
    /// Whether a given string matches this option's definition exactly.
    pub fn matches_exact(&self, s: &OsStr) -> bool {
        argument_matches_exact(self, s)
    }

    /// Whether a given string matches this argument with argument name prefix matching.
    ///
    /// Returns [Some] if yes and the string slice contains the remainder of the value.
    /// [None] means no prefix match.
    pub fn matches_prefix<'a>(&self, s: &'a OsStr) -> Option<Cow<'a, OsStr>> {
        argument_matches_prefix(self, s)
    }
}

/// A collection of options that can be passed to an LLVM program.
///
/// Instances are likely obtained by parsing LLVM tablegen definitions.
pub struct CommandOptions {
    pub options: Vec<ProgramOption>,
}

impl CommandOptions {
    /// Obtain an instance by parsing JSON emitted from `llvm-tblgen`.
    ///
    /// From a LLVM+Clang source checkout, you can obtain this JSON by running
    /// something like
    /// `llvm-tblgen --dump-json clang/include/clang/Driver/Options.td -I llvm/include`.
    pub fn from_json<R: std::io::Read>(reader: R) -> Result<Self, Error> {
        let options: Value = serde_json::from_reader(reader)?;

        let mut program_options = options
            .as_object()
            .ok_or_else(|| Error::JsonParse("JSON input should be an Object".into()))?
            .iter()
            .filter_map(|(k, v)| {
                if k.is_empty() || k.starts_with('!') {
                    None
                } else {
                    Some(serde_json::from_value::<JsonOption>(v.clone()))
                }
            })
            // Remove non-options.
            .filter(|json_option| {
                if let Ok(json_option) = json_option {
                    json_option.super_classes.contains(&"Option".to_string())
                } else {
                    true
                }
            })
            // Filter out empty name, which is weird.
            .filter(|json_option| {
                if let Ok(json_option) = json_option {
                    json_option.name != Some("".to_string())
                } else {
                    false
                }
            })
            .map(|json_option| {
                let json_option = json_option?;

                let kind = OptionKind::try_from(&json_option)?;
                let option_name = json_option.raw_name;
                let name = json_option
                    .name
                    .ok_or_else(|| Error::JsonParse(".Name should be present".into()))?;

                let prefixes = json_option
                    .prefixes
                    .ok_or_else(|| Error::JsonParse(".Prefixes should be present".into()))?
                    .iter()
                    .map(|prefix| ArgumentPrefix::from_str(prefix))
                    .collect::<Result<Vec<_>, Error>>()?;
                let alias = json_option.alias.map(|alias| alias.def);
                let flags = json_option
                    .flags
                    .ok_or_else(|| Error::JsonParse(".Flags should be present".into()))?
                    .into_iter()
                    .map(|v| v.def)
                    .collect::<Vec<_>>();
                let group = json_option.group.map(|group| group.def);

                Ok(ProgramOption {
                    option_name,
                    name,
                    kind,
                    prefixes,
                    alias,
                    flags,
                    group,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // The options sort in descending order so argument parsing isn't
        // ambiguous for joined options that can share a prefix.
        program_options.sort();

        Ok(Self {
            options: program_options,
        })
    }

    /// Iterate over LLVM program option definitions.
    pub fn iter_options(&self) -> impl Iterator<Item = &ProgramOption> {
        self.options.iter()
    }

    /// Obtain all options indexed by their flags.
    ///
    /// Options without flags are not returned.
    pub fn options_by_flag(&self) -> HashMap<&str, Vec<&ProgramOption>> {
        let mut h = HashMap::new();

        for o in &self.options {
            for flag in &o.flags {
                h.entry(flag.as_str()).or_insert_with(Vec::new).push(o);
            }
        }

        h
    }

    /// Obtain all options indexed by their named group.
    ///
    /// Options without a group are not returned.
    pub fn options_by_group(&self) -> HashMap<&str, Vec<&ProgramOption>> {
        let mut h = HashMap::new();

        for o in &self.options {
            if let Some(group) = &o.group {
                h.entry(group.as_str()).or_insert_with(Vec::new).push(o);
            }
        }

        h
    }

    /// Parse an iterable of arguments to a series of options.
    ///
    /// This is how you'll want to parse a command line string into its
    /// internal LLVM options.
    pub fn parse_arguments<I, T>(&self, args: I) -> Result<ParsedArguments, Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut args = args.into_iter();

        let mut parsed_args = vec![];

        while let Some(arg) = self.parse_next_argument(&mut args)? {
            parsed_args.push(arg);
        }

        Ok(ParsedArguments {
            parsed: parsed_args,
        })
    }

    /// Parse the next argument from an iterable of arguments.
    ///
    /// Returns `Ok` if argument iteration did not encounter an error. Returns
    /// `Err` if there was a logical problem with iteration, such as encountering
    /// an argument that wanted values but didn't have any.
    ///
    /// Returns `Some` when an argument was parsed and `None` when no more arguments
    /// are available.
    pub fn parse_next_argument<I, T>(&self, args: &mut I) -> Result<Option<ParsedArgument>, Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let arg = if let Some(arg) = args.next() {
            arg.into()
        } else {
            return Ok(None);
        };

        // TODO expand response files, which have form `@path`.
        // Response files content is treated as regular command line parameters.

        if arg == "-" {
            return Ok(Some(ParsedArgument::Positional(arg)));
        }

        if !arg.to_string_lossy().starts_with('-') {
            return Ok(Some(ParsedArgument::Positional(arg)));
        }

        for definition in &self.options {
            match definition.kind {
                OptionKind::Input => {}
                OptionKind::Unknown => {}
                OptionKind::Flag => {
                    if definition.matches_exact(&arg) {
                        return Ok(Some(ParsedArgument::Flag(definition.clone())));
                    }
                }
                // Joined values look like `name=<value>`.
                OptionKind::Joined => {
                    if let Some(remaining) = definition.matches_prefix(&arg) {
                        return Ok(Some(ParsedArgument::SingleValue(
                            definition.clone(),
                            remaining.to_os_string(),
                        )));
                    }
                }
                OptionKind::CommaJoined => {
                    if let Some(remaining) = definition.matches_prefix(&arg) {
                        return Ok(Some(ParsedArgument::CommaValues(
                            definition.clone(),
                            remaining.to_os_string(),
                        )));
                    }
                }
                // Separate takes value from next argument.
                OptionKind::Separate => {
                    if definition.matches_exact(&arg) {
                        if let Some(value) = args.next() {
                            let value = value.into();

                            return Ok(Some(ParsedArgument::SingleValue(
                                definition.clone(),
                                value,
                            )));
                        } else {
                            return Err(Error::ParseNoArgumentValue(
                                definition.option_name.clone(),
                            ));
                        }
                    }
                }
                // Takes form `-name value` or `-namevalue`. e.g. `-l`.
                OptionKind::JoinedOrSeparate => {
                    if let Some(remaining) = definition.matches_prefix(&arg) {
                        // Empty remaining means we consumed the full argument and the
                        // value is the next argument.
                        if remaining.is_empty() {
                            if let Some(value) = args.next() {
                                let value = value.into();

                                return Ok(Some(ParsedArgument::SingleValue(
                                    definition.clone(),
                                    value,
                                )));
                            } else {
                                return Err(Error::ParseNoArgumentValue(
                                    definition.option_name.clone(),
                                ));
                            }
                        } else {
                            return Ok(Some(ParsedArgument::SingleValue(
                                definition.clone(),
                                remaining.to_os_string(),
                            )));
                        }
                    }
                }

                // Takes form `-name=<key> value`.
                OptionKind::JoinedAndSeparate => {
                    if let Some(remaining) = definition.matches_prefix(&arg) {
                        if let Some(value) = args.next() {
                            let value = value.into();

                            return Ok(Some(ParsedArgument::SingleValueKeyed(
                                definition.clone(),
                                remaining.to_os_string(),
                                value,
                            )));
                        } else {
                            return Err(Error::ParseNoArgumentValue(
                                definition.option_name.clone(),
                            ));
                        }
                    }
                }

                OptionKind::MultiArg(expected_arg_count) => {
                    if definition.matches_exact(&arg) {
                        let values = args
                            .take(expected_arg_count)
                            .map(|x| x.into())
                            .collect::<Vec<_>>();

                        if values.len() != expected_arg_count {
                            return Err(Error::ParseMultipleValuesMissing(
                                definition.option_name.clone(),
                                expected_arg_count,
                                values.len(),
                            ));
                        }

                        return Ok(Some(ParsedArgument::MultipleValues(
                            definition.clone(),
                            values,
                        )));
                    }
                }

                // Consumes all remaining arguments as-is,
                OptionKind::RemainingArgs => {
                    if definition.matches_exact(&arg) {
                        let values = args.map(|x| x.into()).collect::<Vec<_>>();

                        return Ok(Some(ParsedArgument::MultipleValues(
                            definition.clone(),
                            values,
                        )));
                    }
                }

                // Consumes remaining arguments after a joined value.
                OptionKind::RemainingArgsJoined => {
                    if let Some(remaining) = definition.matches_prefix(&arg) {
                        let values = args.map(|x| x.into()).collect::<Vec<_>>();

                        return Ok(Some(ParsedArgument::MultipleValuesKeyed(
                            definition.clone(),
                            remaining.to_os_string(),
                            values,
                        )));
                    }
                }
            }
        }

        Ok(Some(ParsedArgument::Unknown(arg)))
    }
}

/// A parsed argument.
///
/// Instances correspond to a parsed process argument. Some variants correspond
/// to positional or unknown arguments which aren't associated with a given
/// LLVM program option.
///
/// The argument can be derived from 1 or more actual command line arguments:
/// it all depends on the option type.
#[derive(Clone, Debug, PartialEq)]
pub enum ParsedArgument {
    /// Argument is unknown.
    ///
    /// The inner value is the source argument.
    Unknown(OsString),

    /// Argument is a positional argument.
    Positional(OsString),

    /// A flag argument.
    ///
    /// Presence or lack thereof typically conveys boolean state.
    Flag(ProgramOption),

    /// An argument with a single value.
    SingleValue(ProgramOption, OsString),

    /// An argument with a single value keyed to another value.
    SingleValueKeyed(ProgramOption, OsString, OsString),

    /// An argument with comma joined values.
    CommaValues(ProgramOption, OsString),

    /// An argument with multiple values.
    MultipleValues(ProgramOption, Vec<OsString>),

    /// An argument with multiple values keyed to a specific value.
    ///
    /// This is likely used to represent [OptionKind::RemainingArgsJoined].
    MultipleValuesKeyed(ProgramOption, OsString, Vec<OsString>),
}

impl ParsedArgument {
    /// Obtain the [ProgramOption] for this parsed argument, if available.
    pub fn option(&self) -> Option<&ProgramOption> {
        match self {
            Self::Unknown(_) | Self::Positional(_) => None,
            Self::Flag(d) => Some(d),
            Self::SingleValue(d, _) => Some(d),
            Self::SingleValueKeyed(d, _, _) => Some(d),
            Self::CommaValues(d, _) => Some(d),
            Self::MultipleValues(d, _) => Some(d),
            Self::MultipleValuesKeyed(d, _, _) => Some(d),
        }
    }

    /// The clang internal name of this parsed argument, if available.
    pub fn name(&self) -> Option<&str> {
        self.option().map(|d| d.option_name.as_str())
    }

    /// Values for this argument.
    ///
    /// Arguments without values return an empty vec.
    pub fn values(&self) -> Vec<&OsStr> {
        match self {
            Self::Unknown(_) | Self::Positional(_) | Self::Flag(_) => vec![],
            Self::SingleValue(_, value) => vec![value],
            Self::SingleValueKeyed(_, _, value) => vec![value],
            Self::CommaValues(_, value) => vec![value],
            Self::MultipleValues(_, values) => {
                values.iter().map(|x| x.as_os_str()).collect::<Vec<_>>()
            }
            Self::MultipleValuesKeyed(_, _, values) => {
                values.iter().map(|x| x.as_os_str()).collect::<Vec<_>>()
            }
        }
    }

    /// Replace the [ProgramOption] associated with this instance.
    pub fn with_option(self, option: ProgramOption) -> Self {
        match self {
            Self::Unknown(_) | Self::Positional(_) => self,
            Self::Flag(_) => Self::Flag(option),
            Self::SingleValue(_, a) => Self::SingleValue(option, a),
            Self::SingleValueKeyed(_, a, b) => Self::SingleValueKeyed(option, a, b),
            Self::CommaValues(_, a) => Self::CommaValues(option, a),
            Self::MultipleValues(_, a) => Self::MultipleValues(option, a),
            Self::MultipleValuesKeyed(_, a, b) => Self::MultipleValuesKeyed(option, a, b),
        }
    }
}

/// Represents a collection of parsed command line arguments.
#[derive(Clone, Debug)]
pub struct ParsedArguments {
    parsed: Vec<ParsedArgument>,
}

impl ParsedArguments {
    /// Obtain a reference to the raw [ParsedArgument] list.
    pub fn parsed(&self) -> &Vec<ParsedArgument> {
        &self.parsed
    }

    /// Obtain an iterable over [ParsedArgument].
    pub fn iter_parsed(&self) -> impl Iterator<Item = &ParsedArgument> {
        self.parsed.iter()
    }

    /// Resolve aliases to their canonical options.
    ///
    /// If an internal [ParsedArgument] is an alias, it will be resolved to its
    /// canonical [ProgramOption].
    pub fn resolve_aliases(self, options: &CommandOptions) -> Result<Self, Error> {
        let parsed = self
            .parsed
            .into_iter()
            .map(|arg| {
                if let Some(option) = arg.option() {
                    if let Some(alias) = &option.alias {
                        if let Some(canonical) = options
                            .iter_options()
                            .find(|candidate| &candidate.option_name == alias)
                        {
                            Ok(arg.with_option(canonical.clone()))
                        } else {
                            Err(Error::AliasMissing(
                                option.option_name.clone(),
                                alias.to_string(),
                            ))
                        }
                    } else {
                        Ok(arg)
                    }
                } else {
                    Ok(arg)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { parsed })
    }
}
