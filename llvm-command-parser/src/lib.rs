// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/*! Option parsing for LLVM commands.

This crate provides ready-to-use option parsers for LLVM commands. It does
so by parsing LLVM tablegen JSON data provided by the
[llvm_command_tablegen_json] crate.
*/

use {llvm_command_tablegen_json::LLVM_13, llvm_option_parser::CommandOptions};

/// Obtain [CommandOptions] for a named command in LLVM version 13.
///
/// Tablegen JSON data for LLVM commands is embedded in the crate and
/// available to be parsed at run-time. Calling this function will trigger
/// the parsing of this data for the given command.
pub fn llvm_13_options(command: &str) -> Option<CommandOptions> {
    if let Some(data) = LLVM_13.get(command) {
        let cursor = std::io::Cursor::new(data);

        let options =
            CommandOptions::from_json(cursor).expect("built-in JSON should parse successfully");

        Some(options)
    } else {
        None
    }
}

/// Obtain LLVM option definitions for Clang version 13.
#[cfg(feature = "13-clang")]
pub fn clang_13_options() -> CommandOptions {
    llvm_13_options("clang").expect("clang options should be available")
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "13-clang")]
    use llvm_option_parser::{Error, ParsedArgument};

    #[test]
    fn parse_all() {
        for command in LLVM_13.keys() {
            let options = llvm_13_options(command).unwrap();
            options.options_by_group();
            options.options_by_flag();
        }
    }

    #[cfg(feature = "13-clang")]
    #[test]
    fn clang_13() -> Result<(), Error> {
        let options = clang_13_options();

        assert_eq!(options.options[0].option_name, "C");
        assert_eq!(options.options[1].option_name, "CC");
        assert_eq!(options.options.last().unwrap().option_name, "y");

        Ok(())
    }

    #[cfg(feature = "13-clang")]
    #[test]
    fn parse_arg_flavors() -> Result<(), Error> {
        let options = clang_13_options();

        // Positional argument.
        let args = options.parse_arguments(vec!["clang"])?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0], ParsedArgument::Positional("clang".into()));

        // A flag argument with a single dash.
        let args = options.parse_arguments(vec!["-pthread"])?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0].name(), Some("pthread"));

        // Joined with a single dash.
        let args = options.parse_arguments(vec!["-Wno-unused-result"])?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0].name(), Some("W_Joined"));
        assert!(matches!(
            args.parsed()[0],
            ParsedArgument::SingleValue(_, _)
        ));
        assert_eq!(args.parsed()[0].values(), vec!["no-unused-result"]);

        // Joined with equals value.
        let args = options.parse_arguments(vec!["-fvisibility=hidden"])?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0].name(), Some("fvisibility_EQ"));
        assert!(matches!(
            args.parsed()[0],
            ParsedArgument::SingleValue(_, _)
        ));
        assert_eq!(args.parsed()[0].values(), vec!["hidden"]);

        // Joined or separate joined flavor.
        let args = options.parse_arguments(vec!["-DDEBUG"])?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0].name(), Some("D"));
        assert!(matches!(
            args.parsed()[0],
            ParsedArgument::SingleValue(_, _)
        ));
        assert_eq!(args.parsed()[0].values(), vec!["DEBUG"]);

        // Joined or separate separate flavor.
        let args = options.parse_arguments(vec!["-D", "DEBUG"])?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0].name(), Some("D"));
        assert!(matches!(
            args.parsed()[0],
            ParsedArgument::SingleValue(_, _)
        ));
        assert_eq!(args.parsed()[0].values(), vec!["DEBUG"]);

        // Separate.
        let args = options.parse_arguments(vec!["-target", "value"])?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0].name(), Some("target_legacy_spelling"));
        assert!(matches!(
            args.parsed()[0],
            ParsedArgument::SingleValue(_, _)
        ));
        assert_eq!(args.parsed()[0].values(), vec!["value"]);
        // -target is an alias. Check that it resolves.
        let args = args.resolve_aliases(&options)?;
        assert_eq!(args.parsed().len(), 1);
        assert_eq!(args.parsed()[0].name(), Some("target"));
        assert!(matches!(
            args.parsed()[0],
            ParsedArgument::SingleValue(_, _)
        ));
        assert_eq!(args.parsed()[0].values(), vec!["value"]);

        Ok(())
    }
}
