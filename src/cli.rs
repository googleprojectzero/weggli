/*
 Copyright 2021 Google LLC

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

      https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 */

use std::{path::{Path, PathBuf}};
use clap::{App, Arg};
use simplelog::*;

pub struct Args {
    pub path: PathBuf,
    pub pattern: Vec<String>,
    pub before: usize,
    pub after: usize,
    pub extensions: Vec<String>,
    pub regexes: Vec<String>,
    pub limit: bool,
    pub cpp: bool,
    pub unique: bool,
    pub force_color: bool,
    pub force_query: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// Parse command arguments and return them inside the Args structure.
/// The clap crate handles program exit and error messages for invalid arguments.
pub fn parse_arguments() -> Args {
    let matches = App::new("weggli")
        .version("0.2.3")
        .author("Felix Wilhelm <fwilhelm@google.com>")
        .about(help::ABOUT)
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .help_template(help::TEMPLATE)
        .arg(
            Arg::new("PATTERN")
                .help("Search pattern.")
                .long_help(help::PATTERN)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("p")
                .long("pattern")
                .short('p')
                .help("Specify additional search patterns.")
                .takes_value(true)
                .multiple_values(true)
                .number_of_values(1),
        )
        .arg(
            Arg::new("PATH")
                .help("A file or directory to search.")
                .long_help(help::PATH)
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new("v")
                .long("verbose")
                .short('v')
                .multiple_occurrences(true)
                .help("Sets the level of verbosity."),
        )
        .arg(
            Arg::new("extensions")
                .long("extensions")
                .short('e')
                .takes_value(true)
                .multiple_values(true)
                .help("File extensions to include in the search."),
        )
        .arg(
            Arg::new("before")
                .long("before")
                .short('B')
                .takes_value(true)
                .help("Lines to print before a match. Default = 5."),
        )
        .arg(
            Arg::new("after")
                .long("after")
                .short('A')
                .takes_value(true)
                .help("Lines to print after a match. Default = 5."),
        )
        .arg(
            Arg::new("limit")
                .long("limit")
                .short('l')
                .takes_value(false)
                .help("Only show the first match in each function."),
        )
        .arg(
            Arg::new("regex")
                .long("regex")
                .short('R')
                .takes_value(true)
                .multiple_occurrences(true)
                .number_of_values(1)
                .help("Enforce that a variable has to (not) match a regex.")
                .long_help(help::REGEX),
        )
        .arg(
            Arg::new("cpp")
                .short('X')
                .long("cpp")
                .takes_value(false)
                .help("Enable C++ mode."),
        )
        .arg(
            Arg::new("color")
                .short('C')
                .long("color")
                .takes_value(false)
                .help("Force enable color output."),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .short('f')
                .takes_value(false)
                .help("Force a search even if the queries contains syntax errors."),
        )
        .arg(
            Arg::new("unique")
                .long("unique")
                .short('u')
                .takes_value(false)
                .help("Enforce uniqueness of variable matches.")
                .long_help(help::UNIQUE),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .takes_value(true)
                .multiple_values(true)
                .help("Exclude files that match the given regex."),
        )
        .arg(
            Arg::new("include")
                .long("include")
                .takes_value(true)
                .multiple_values(true)
                .help("Only search files that match the given regex."),
        )
        .get_matches();

    let helper = |option_name| -> Vec<String> {
        if let Some(v) = matches.values_of(option_name) {
            v.map(|v| v.to_string()).collect()
        } else {
            vec![]
        }
    };

    let level = match matches.occurrences_of("v") {
        0 => LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        _ => log::LevelFilter::Debug,
    };

    let _ = SimpleLogger::init(level, Config::default());

    let directory = Path::new(matches.value_of("PATH").unwrap_or("."));

    let mut pattern = vec![matches.value_of("PATTERN").unwrap().to_string()];
    if let Some(p) = matches.values_of("p") {
        pattern.extend(p.map(|v| v.to_string()))
    }

    let regexes = helper("regex");

    let path = if directory.is_absolute() || directory.to_string_lossy() == "-" {
        directory.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(directory)
    };

    let before = match matches.value_of("before") {
        Some(v) => v.parse().unwrap_or(5),
        None => 5,
    };

    let after = match matches.value_of("after") {
        Some(v) => v.parse().unwrap_or(5),
        None => 5,
    };

    let limit = matches.occurrences_of("limit") > 0;

    let unique = matches.occurrences_of("unique") > 0;

    let cpp = matches.occurrences_of("cpp") > 0;
    let force_color = matches.occurrences_of("color") > 0;

    let extensions = {
        let e = helper("extensions");
        if e.is_empty() {
            if !cpp {
                vec!["c".to_string(), "h".into()]
            } else {
                vec![
                    "cc".to_string(),
                    "cpp".into(),
                    "h".into(),
                    "cxx".into(),
                    "hpp".into(),
                ]
            }
        } else {
            e
        }
    };

    let exclude = helper("exclude");
    let include = helper("include");

    let force_query = matches.occurrences_of("force") > 0;

    Args {
        path,
        pattern,
        before,
        after,
        extensions,
        regexes,
        limit,
        cpp,
        unique,
        force_color,
        force_query,
        include,
        exclude,
    }
}


mod help {
 pub const ABOUT: &str = "\
 weggli is a semantic search tool for C and C++ codebases.
 It is designed to quickly find interesting code pattern in large codebases.
 
 Use -h for short descriptions and --help for more details.
 
 Homepage: https://github.com/googleprojectzero/weggli";
 
 pub const TEMPLATE: &str = "\
 {bin} {version}
 {author}
 
 {about}
 
 USAGE: {usage}
 
 ARGS:
 {positionals}
 
 OPTIONS:
 {options}";
 
 pub const PATTERN: &str = "\
 A weggli search pattern. weggli's query language closely resembles
 C and C++ with a small number of extra features.
 
 For example, the pattern '{_ $buf[_]; memcpy($buf,_,_);}' will
 find all calls to memcpy that directly write into a stack buffer.
 
 Besides normal C and C++ constructs, weggli's query language
 supports the following features:
 
 _        Wildcard. Will match on any AST node. 
 
 $var     Variables. Can be used to write queries that are independent
          of identifiers. Variables match on identifiers, types,
          field names or namespaces. The --unique option
          optionally enforces that $x != $y != $z. The --regex option can
          enforce that the variable has to match (or not match) a
          regular expression.
 
 _(..)    Subexpressions. The _(..) wildcard matches on arbitrary
          sub expressions. This can be helpful if you are looking for some
          operation involving a variable, but don't know more about it.
          For example, _(test) will match on expressions like test+10,
          buf[test->size] or f(g(&test));
 
 not:     Negative sub queries. Only show results that do not match the
          following sub query. For example, '{not: $fv==NULL; not: $fv!=NULL *$v;}'
          would find pointer dereferences that are not preceded by a NULL check.

strict:   Enable stricter matching. This turns off statement unwrapping and greedy
          function name matching. For example 'strict: func();' will not match
          on 'if (func() == 1)..' or 'a->func()' anymore. 
 
 weggli automatically unwraps expression statements in the query source 
 to search for the inner expression instead. This means that the query `{func($x);}` 
 will match on `func(a);`, but also on `if (func(a)) {..}` or  `return func(a)`. 
 Matching on `func(a)` will also match on `func(a,b,c)` or `func(z,a)`. 
 Similarly, `void func($t $param)` will also match function definitions 
 with multiple parameters. 
 
 Additional patterns can be specified using the --pattern (-p) option. This makes
 it possible to search across functions or type definitions.
 ";
 
 pub const PATH: &str = "\
 Input directory or file to search. By default, weggli will search inside 
 .c and .h files for the default C mode or .cc, .cpp, .cxx, .h and .hpp files when
 executing in C++ mode (using the --cpp option).
 Alternative file endings can be specified using the --extensions (-e) option.
 
 When combining weggli with other tools or preprocessing steps, 
 files can also be specified via STDIN by setting the directory to '-' 
 and piping a list of filenames.
 ";
 
 pub const REGEX: &str = "\
 Filter variable matches based on a regular expression. 
 This feature uses the Rust regex crate, so most Perl-style
 regular expression features are supported.
 (see https://docs.rs/regex/1.5.4/regex/#syntax)
 
 Examples:
 
 Find calls to functions starting with the string 'mem':
 weggli -R 'func=^mem' '$func(_);'       
 
 Find memcpy calls where the last argument is NOT named 'size':
 weggli -R 's!=^size$' 'memcpy(_,_,$s);' 
 ";
 
 pub const UNIQUE: &str = "\
 Enforce uniqueness of variable matches.
 By default, two variables such as $a and $b can match on identical values.
 For example, the query '$x=malloc($a); memcpy($x, _, $b);' would
 match on both
 
 void *buf = malloc(size);
 memcpy(buf, src, size);
 
 and
 
 void *buf = malloc(some_constant);
 memcpy(buf, src, size);
 
 Using the unique flag would filter out the first match as $a==$b.
 ";
} 