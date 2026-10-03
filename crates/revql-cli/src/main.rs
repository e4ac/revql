mod args;
mod error;

use crate::args::HttpMethod;
use crate::error::AppError;
use clap::Parser;
use colored::{Color, Colorize};
use revql_core::network::{get_network_content, post_network_content};
use revql_core::path::{AllPaths, Path, PathStep, format_path};
use revql_core::schema::{MatchConfig, TypeMap, get_schema_type_map, parse_schema};
use revql_core::search::{find_path_lines, get_end_types, search_paths};
use std::fs::File;
use std::io::{BufWriter, Write};

fn colour_step(step: &PathStep) -> String {
    format!("{}.{}", step.from_type.color(Color::Red), step.from_field.color(Color::Blue))
}

fn get_path_str(path: &Path, color: bool) -> String {
    if color {
        path.iter().map(colour_step).collect::<Vec<_>>().join(" -> ")
    } else {
        format_path(path)
    }
}

fn print_colour_paths(paths: &AllPaths, end_type: &str, color: bool) {
    for path in paths {
        let path_str = get_path_str(path, color);
        if !path_str.is_empty() {
            println!("{end_type}: {path_str}");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = args::Arguments::parse();
    let query = if let Some(file) = args.query_opt.query_file {
        Some(std::fs::read_to_string(file)?)
    } else {
        args.query_opt.query
    };

    let content = if args.query_opt.url {
        if args.query_opt.method == HttpMethod::Post {
            post_network_content(args.file.as_ref(), &args.query_opt.headers, query).await?
        } else {
            get_network_content(args.file.as_ref(), &args.query_opt.headers, query).await?
        }
    } else {
        std::fs::read_to_string(&args.file)?
    };

    let schema = parse_schema(content.as_str())?;
    let config = MatchConfig {
        contains: args.match_opt.contains,
        ignore_case: args.match_opt.ignore_case,
    };

    let mut writer = if let Some(filename) = &args.output_file {
        let file = File::create(filename)?;
        Some(BufWriter::new(file))
    } else {
        None
    };

    if let Some(w) = writer.as_mut() {
        for line in find_path_lines(&schema, &args.search, args.search_mode.into(), config) {
            writeln!(w, "{line}")?;
        }
        w.flush()?;
    } else {
        let type_map = get_schema_type_map(&schema).collect::<TypeMap>();
        let end_types = get_end_types(&schema, &args.search, args.search_mode.into(), config);
        for (end, paths) in search_paths(&["Query", "Mutation"], end_types.as_slice(), &type_map) {
            print_colour_paths(&paths, end, !args.no_color);
        }
    }
    Ok(())
}
