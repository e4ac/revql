use crate::path::{AllPaths, find_all_paths, format_path};
use crate::schema::{
    MatchConfig, TypeMap, get_schema_type_map, get_schema_types_by_description, get_schema_types_by_field,
    get_schema_types_by_name,
};
use graphql_introspection_query::introspection_response::Schema;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SearchMode {
    #[default]
    Type,
    Description,
    Field,
}

#[inline]
pub fn search_paths<'a>(
    start_types: &'a [&'a str],
    end_types: &'a [&'a str],
    type_map: &'a TypeMap,
) -> impl Iterator<Item = (&'a str, AllPaths)> {
    start_types
        .iter()
        .flat_map(|start| end_types.iter().map(|end| (*end, find_all_paths(type_map, start, end))))
}

#[inline]
#[must_use]
pub fn get_end_types<'a>(schema: &'a Schema, search: &str, mode: SearchMode, config: MatchConfig) -> Vec<&'a str> {
    match mode {
        SearchMode::Field => get_schema_types_by_field(schema, search, config)
            .filter_map(|end| end.name.as_deref())
            .collect(),
        SearchMode::Description => get_schema_types_by_description(schema, search, config)
            .filter_map(|end| end.name.as_deref())
            .collect(),
        SearchMode::Type => get_schema_types_by_name(schema, search, config)
            .filter_map(|end| end.name.as_deref())
            .collect(),
    }
}

#[must_use]
pub fn find_path_lines(schema: &Schema, search: &str, mode: SearchMode, config: MatchConfig) -> Vec<String> {
    let type_map = get_schema_type_map(schema).collect::<TypeMap>();
    let end_types = get_end_types(schema, search, mode, config);
    search_paths(&["Query", "Mutation"], end_types.as_slice(), &type_map)
        .flat_map(|(end, paths)| {
            paths.into_iter().filter_map(move |path| {
                let path_str = format_path(&path);
                (!path_str.is_empty()).then(|| format!("{end}: {path_str}"))
            })
        })
        .collect()
}
