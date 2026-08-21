mod args;
mod error;
mod network;
mod path;
mod schema;

use crate::args::{HttpMethod, SearchMode};
use crate::error::AppError;
use crate::network::{get_network_content, post_network_content};
use crate::path::{AllPaths, Path, find_all_paths};
use crate::schema::{
    MatchConfig, TypeMap, get_schema_type_map, get_schema_types_by_description, get_schema_types_by_field,
    get_schema_types_by_name,
};
use clap::Parser;
use graphql_introspection_query::introspection_response::{IntrospectionResponse, Schema};
use std::fs::File;
use std::io::{BufWriter, Write};

fn parse_schema(json: &str) -> Result<Schema, AppError> {
    let result = serde_json::from_str::<IntrospectionResponse>(json)?;
    result.into_schema().schema.ok_or(AppError::Schema("Invalid schema"))
}

fn search_paths<'a>(
    start_types: &'a [&'a str],
    end_types: &'a [&'a str],
    type_map: &'a TypeMap,
) -> impl Iterator<Item = (&'a str, AllPaths)> {
    start_types
        .iter()
        .flat_map(|start| end_types.iter().map(|end| (*end, find_all_paths(type_map, start, end))))
}

fn get_path_str(path: &Path, color: bool) -> String {
    path.iter().map(|step| step.as_string(color)).collect::<Vec<_>>().join(" -> ")
}

fn print_colour_paths(paths: &AllPaths, end_type: &str, color: bool) {
    for path in paths {
        let path_str = get_path_str(path, color);
        if !path_str.is_empty() {
            println!("{end_type}: {path_str}");
        }
    }
}

fn write_output(paths: &AllPaths, end_type: &str, writer: &mut impl Write) -> Result<(), AppError> {
    for path in paths {
        let path_str = get_path_str(path, false);
        if !path_str.is_empty() {
            writeln!(writer, "{end_type}: {path_str}")?;
        }
    }
    Ok(())
}

fn get_end_types<'a>(schema: &'a Schema, search: &str, mode: SearchMode, config: MatchConfig) -> Vec<&'a str> {
    if mode == SearchMode::Field {
        get_schema_types_by_field(schema, search, config)
            .filter_map(|end| end.name.as_deref())
            .collect()
    } else if mode == SearchMode::Description {
        get_schema_types_by_description(schema, search, config)
            .filter_map(|end| end.name.as_deref())
            .collect()
    } else {
        get_schema_types_by_name(schema, search, config)
            .filter_map(|end| end.name.as_deref())
            .collect()
    }
}

fn main() -> Result<(), AppError> {
    let args = args::Arguments::parse();
    let query = if let Some(file) = args.query_opt.query_file {
        Some(std::fs::read_to_string(file)?)
    } else {
        args.query_opt.query
    };

    let content = if args.query_opt.url {
        if args.query_opt.method == HttpMethod::Post {
            post_network_content(args.file.as_ref(), &args.query_opt.headers, query)?
        } else {
            get_network_content(args.file.as_ref(), &args.query_opt.headers, query)?
        }
    } else {
        std::fs::read_to_string(&args.file)?
    };

    let schema = parse_schema(content.as_str())?;
    let config = MatchConfig {
        contains: args.match_opt.contains,
        ignore_case: args.match_opt.ignore_case,
    };

    let type_map = get_schema_type_map(&schema).collect::<TypeMap>();
    let end_types = get_end_types(&schema, &args.search, args.search_mode, config);
    let mut writer = if let Some(filename) = &args.output_file {
        let file = File::create(filename)?;
        Some(BufWriter::new(file))
    } else {
        None
    };

    for (end, paths) in search_paths(&["Query", "Mutation"], end_types.as_slice(), &type_map) {
        if let Some(w) = writer.as_mut() {
            write_output(&paths, end, w)?;
        } else {
            print_colour_paths(&paths, end, !args.no_color);
        }
    }

    if let Some(w) = writer.as_mut() {
        w.flush()?;
    }
    Ok(())
}
