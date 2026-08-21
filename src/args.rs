use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
pub struct Arguments {
    /// Specifies the path to the JSON file containing the introspection result.
    #[clap(required = true)]
    pub file: String,

    /// Specifies the search term.
    #[clap(required = true)]
    pub search: String,

    /// Specifies the output file path. Print to console if not specified.
    #[clap(short, long)]
    pub output_file: Option<String>,

    /// Specifies whether to disable color output.
    #[clap(long)]
    pub no_color: bool,

    /// Specifies the search mode.
    #[clap(long, default_value = "type", ignore_case = true)]
    pub search_mode: SearchMode,

    #[clap(flatten)]
    pub match_opt: MatchOptions,

    #[clap(flatten)]
    pub query_opt: QueryOptions,
}

#[derive(Copy, Clone, Debug, Parser)]
pub struct MatchOptions {
    /// Specifies whether to perform a "contains" match instead of an exact match.
    #[clap(long)]
    pub contains: bool,

    /// Specifies whether to ignore cases when searching.
    #[clap(long)]
    pub ignore_case: bool,
}

#[derive(Clone, Debug, Parser)]
pub struct QueryOptions {
    /// Specifies that <FILE> is a URL instead of a schema file.
    #[clap(short, long)]
    pub url: bool,

    /// Specifies the header to include in the request.
    #[clap(short = 'H', long = "header", action = clap::ArgAction::Append)]
    pub headers: Vec<String>,

    /// Specifies the path to a custom `.graphql` file for the introspection query.
    #[clap(long)]
    pub query_file: Option<PathBuf>,

    /// Specifies a custom introspection query.
    #[clap(long)]
    pub query: Option<String>,

    /// Specifies the HTTP method to use.
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
pub enum SearchMode {
    #[default]
    Type,
    Description,
    Field,
}
