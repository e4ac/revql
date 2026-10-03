use clap::{Parser, ValueEnum};
use revql_core::search::SearchMode;
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
pub struct Arguments {
    /// The path to the JSON file containing the introspection result.
    #[clap(required = true)]
    pub file: String,

    /// The search term.
    #[clap(required = true)]
    pub search: String,

    /// The output file path. Print to console if not specified.
    #[clap(short, long)]
    pub output_file: Option<String>,

    /// Whether to disable color output.
    #[clap(long)]
    pub no_color: bool,

    /// The search mode.
    #[clap(long, default_value = "type", ignore_case = true)]
    pub search_mode: SearchModeArg,

    #[clap(flatten)]
    pub match_opt: MatchOptions,

    #[clap(flatten)]
    pub query_opt: QueryOptions,
}

#[derive(Copy, Clone, Debug, Parser)]
pub struct MatchOptions {
    /// Whether to perform a "contains" match instead of an exact match.
    #[clap(long)]
    pub contains: bool,

    /// Whether to ignore cases when searching.
    #[clap(long)]
    pub ignore_case: bool,
}

#[derive(Clone, Debug, Parser)]
pub struct QueryOptions {
    /// Specifies that <FILE> is a URL instead of a schema file.
    #[clap(short, long)]
    pub url: bool,

    /// The header to include in the request.
    #[clap(short = 'H', long = "header", action = clap::ArgAction::Append)]
    pub headers: Vec<String>,

    /// The path to a custom `.graphql` file for the introspection query.
    #[clap(long)]
    pub query_file: Option<PathBuf>,

    /// The custom introspection query.
    #[clap(long)]
    pub query: Option<String>,

    /// The HTTP method to use.
    #[clap(short = 'X', long = "request", default_value = "POST", ignore_case = true)]
    pub method: HttpMethod,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    #[default]
    Post,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SearchModeArg {
    #[default]
    Type,
    Description,
    Field,
}

impl From<SearchModeArg> for SearchMode {
    #[inline]
    fn from(mode: SearchModeArg) -> Self {
        match mode {
            SearchModeArg::Type => Self::Type,
            SearchModeArg::Description => Self::Description,
            SearchModeArg::Field => Self::Field,
        }
    }
}
