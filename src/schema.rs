use graphql_introspection_query::introspection_response::{__TypeKind, FullType, Schema, TypeRef};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Default)]
pub struct MatchConfig {
    pub contains: bool,
    pub ignore_case: bool,
}

impl MatchConfig {
    pub fn match_str(self, a: &str, b: &str) -> bool {
        match (self.contains, self.ignore_case) {
            (false, false) => a == b,
            (true, false) => a.contains(b),
            (false, true) => a.to_lowercase() == b.to_lowercase(),
            (true, true) => a.to_lowercase().contains(b.to_lowercase().as_str()),
        }
    }
}

pub type TypeMap<'a> = HashMap<&'a str, &'a FullType>;

pub fn get_schema_types(schema: &Schema) -> impl Iterator<Item = &FullType> {
    schema.types.as_deref().into_iter().flatten().flatten().map(|t| &t.full_type)
}

pub fn get_schema_type_map(schema: &Schema) -> impl Iterator<Item = (&str, &FullType)> {
    get_schema_types(schema).filter_map(|t| Some((t.name.as_deref()?, t)))
}

pub fn get_schema_types_by_name<'a>(schema: &'a Schema, name: &str, config: MatchConfig) -> impl Iterator<Item = &'a FullType> {
    get_schema_types(schema).filter(move |t| {
        let Some(type_name) = t.name.as_deref() else {
            return false;
        };
        config.match_str(type_name, name)
    })
}

pub fn get_schema_types_by_field<'a>(schema: &'a Schema, name: &str, config: MatchConfig) -> impl Iterator<Item = &'a FullType> {
    get_schema_types(schema).filter(move |t| {
        let Some(fields) = t.fields.as_deref() else {
            return false;
        };

        fields.iter().any(|f| {
            let Some(field_name) = f.name.as_deref() else {
                return false;
            };
            config.match_str(field_name, name)
        })
    })
}

pub fn get_schema_types_by_description<'a>(
    schema: &'a Schema,
    name: &str,
    config: MatchConfig,
) -> impl Iterator<Item = &'a FullType> {
    get_schema_types(schema).filter(move |t| {
        let matches = t.description.as_deref().is_some_and(|d| config.match_str(d, name));
        if matches {
            return true;
        }

        t.fields.as_deref().is_some_and(|fields| {
            fields
                .iter()
                .any(|f| f.description.as_deref().is_some_and(|d| config.match_str(d, name)))
        })
    })
}

pub fn unwrap_type_name(type_ref: &TypeRef) -> Option<&str> {
    if let Some(name) = &type_ref.name {
        return Some(name.as_str());
    }
    if let Some(inner) = &type_ref.of_type {
        return unwrap_type_name(inner);
    }
    None
}

pub const fn is_composite(t: &FullType) -> bool {
    matches!(t.kind, Some(__TypeKind::OBJECT | __TypeKind::INTERFACE | __TypeKind::UNION))
}

#[allow(dead_code)] // planned to use in future to remove relay nodes from output
pub fn is_relay(t: &FullType) -> bool {
    let Some(type_name) = t.name.as_deref() else {
        return false;
    };
    let has_field = |name: &str| {
        t.fields
            .as_ref()
            .into_iter()
            .flatten()
            .any(|f| f.name.as_deref() == Some(name))
    };

    if type_name == "PageInfo" {
        return has_field("hasNextPage") && has_field("hasPreviousPage");
    }
    if type_name.ends_with("Connection") {
        return has_field("edges") && has_field("pageInfo");
    }
    if type_name.ends_with("Edge") {
        return has_field("cursor") && has_field("node");
    }
    false
}

#[cfg(test)]
mod match_config_tests {
    use super::*;

    fn create_config(contains: bool, ignore_case: bool) -> MatchConfig {
        MatchConfig { contains, ignore_case }
    }

    #[test]
    fn test_match_str_exact_case_sensitive() {
        let config = create_config(false, false);
        assert!(config.match_str("GraphQL", "GraphQL"));
        assert!(!config.match_str("GraphQL", "graphql"));
        assert!(!config.match_str("GraphQL", "Graph"));
        assert!(!config.match_str("Graph", "GraphQL"));
    }

    #[test]
    fn test_match_str_substring_case_sensitive() {
        let config = create_config(true, false);
        assert!(config.match_str("UserSchema", "Schema"));
        assert!(config.match_str("UserSchema", "User"));
        assert!(!config.match_str("UserSchema", "schema"));
        assert!(!config.match_str("UserSchema", "Admin"));
    }

    #[test]
    fn test_match_str_exact_case_insensitive() {
        let config = create_config(false, true);
        assert!(config.match_str("GraphQL", "graphql"));
        assert!(config.match_str("graphql", "GRAPHQL"));
        assert!(!config.match_str("GraphQL", "graph"));
    }

    #[test]
    fn test_match_str_substring_case_insensitive() {
        let config = create_config(true, true);
        assert!(config.match_str("UserSchema", "schema"));
        assert!(config.match_str("userschema", "SCHEMA"));
        assert!(config.match_str("UserSchema", "User"));
        assert!(!config.match_str("UserSchema", "Admin"));
    }

    #[test]
    fn test_match_str_empty_strings() {
        assert!(create_config(true, false).match_str("User", ""));
        assert!(create_config(true, true).match_str("User", ""));
        assert!(create_config(false, false).match_str("", ""));
        assert!(create_config(false, true).match_str("", ""));
        assert!(!create_config(false, false).match_str("User", ""));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphql_introspection_query::introspection_response::{__TypeKind, FullTypeFields, FullTypeFieldsType};

    #[test]
    fn test_unwrap_type_name() {
        let type_ref = TypeRef {
            kind: Some(__TypeKind::OBJECT),
            name: Some("User".to_owned()),
            of_type: None,
        };
        assert_eq!(unwrap_type_name(&type_ref), Some("User"));
    }

    #[test]
    fn test_unwrap_type_name_empty() {
        let type_ref = TypeRef {
            kind: Some(__TypeKind::OBJECT),
            name: None,
            of_type: None,
        };
        assert_eq!(unwrap_type_name(&type_ref), None);
    }

    #[test]
    fn test_unwrap_type_name_nested() {
        let type_ref = TypeRef {
            kind: Some(__TypeKind::NON_NULL),
            name: None,
            of_type: Some(Box::new(TypeRef {
                kind: Some(__TypeKind::LIST),
                name: None,
                of_type: Some(Box::new(TypeRef {
                    kind: Some(__TypeKind::NON_NULL),
                    name: None,
                    of_type: Some(Box::new(TypeRef {
                        kind: Some(__TypeKind::OBJECT),
                        name: Some("User".to_owned()),
                        of_type: None,
                    })),
                })),
            })),
        };
        assert_eq!(unwrap_type_name(&type_ref), Some("User"));
    }

    #[test]
    fn test_is_composite() {
        let create = |kind| FullType {
            kind: Some(kind),
            name: None,
            description: None,
            fields: None,
            input_fields: None,
            interfaces: None,
            enum_values: None,
            possible_types: None,
        };

        assert!(is_composite(&create(__TypeKind::OBJECT)));
        assert!(is_composite(&create(__TypeKind::INTERFACE)));
        assert!(is_composite(&create(__TypeKind::UNION)));
        assert!(!is_composite(&create(__TypeKind::SCALAR)));
        assert!(!is_composite(&create(__TypeKind::ENUM)));
        assert!(!is_composite(&create(__TypeKind::INPUT_OBJECT)));
        assert!(!is_composite(&create(__TypeKind::LIST)));
        assert!(!is_composite(&create(__TypeKind::NON_NULL)));
        assert!(!is_composite(&create(__TypeKind::Other("TestType".to_owned()))));
    }

    #[test]
    fn test_is_relay_page_info() {
        let relay = FullType {
            kind: Some(__TypeKind::OBJECT),
            name: Some("PageInfo".to_owned()),
            description: None,
            fields: Some(vec![
                FullTypeFields {
                    name: Some("hasNextPage".to_owned()),
                    description: None,
                    args: None,
                    type_: Some(FullTypeFieldsType {
                        type_ref: TypeRef {
                            kind: Some(__TypeKind::SCALAR),
                            name: Some("Boolean".to_owned()),
                            of_type: None,
                        },
                    }),
                    is_deprecated: None,
                    deprecation_reason: None,
                },
                FullTypeFields {
                    name: Some("hasPreviousPage".to_owned()),
                    description: None,
                    args: None,
                    type_: Some(FullTypeFieldsType {
                        type_ref: TypeRef {
                            kind: Some(__TypeKind::SCALAR),
                            name: Some("Boolean".to_owned()),
                            of_type: None,
                        },
                    }),
                    is_deprecated: None,
                    deprecation_reason: None,
                },
            ]),
            input_fields: None,
            interfaces: None,
            enum_values: None,
            possible_types: None,
        };
        assert!(is_relay(&relay));
    }

    #[test]
    fn test_is_relay_connection() {
        let relay = FullType {
            kind: Some(__TypeKind::OBJECT),
            name: Some("UserConnection".to_owned()),
            description: None,
            fields: Some(vec![
                FullTypeFields {
                    name: Some("edges".to_owned()),
                    description: None,
                    args: None,
                    type_: Some(FullTypeFieldsType {
                        type_ref: TypeRef {
                            kind: Some(__TypeKind::SCALAR),
                            name: Some("UserEdge".to_owned()),
                            of_type: None,
                        },
                    }),
                    is_deprecated: None,
                    deprecation_reason: None,
                },
                FullTypeFields {
                    name: Some("pageInfo".to_owned()),
                    description: None,
                    args: None,
                    type_: Some(FullTypeFieldsType {
                        type_ref: TypeRef {
                            kind: Some(__TypeKind::SCALAR),
                            name: Some("PageInfo".to_owned()),
                            of_type: None,
                        },
                    }),
                    is_deprecated: None,
                    deprecation_reason: None,
                },
            ]),
            input_fields: None,
            interfaces: None,
            enum_values: None,
            possible_types: None,
        };
        assert!(is_relay(&relay));
    }

    #[test]
    fn test_is_relay_edge() {
        let relay = FullType {
            kind: Some(__TypeKind::OBJECT),
            name: Some("UserEdge".to_owned()),
            description: None,
            fields: Some(vec![
                FullTypeFields {
                    name: Some("cursor".to_owned()),
                    description: None,
                    args: None,
                    type_: Some(FullTypeFieldsType {
                        type_ref: TypeRef {
                            kind: Some(__TypeKind::SCALAR),
                            name: Some("String".to_owned()),
                            of_type: None,
                        },
                    }),
                    is_deprecated: None,
                    deprecation_reason: None,
                },
                FullTypeFields {
                    name: Some("node".to_owned()),
                    description: None,
                    args: None,
                    type_: Some(FullTypeFieldsType {
                        type_ref: TypeRef {
                            kind: Some(__TypeKind::OBJECT),
                            name: Some("User".to_owned()),
                            of_type: None,
                        },
                    }),
                    is_deprecated: None,
                    deprecation_reason: None,
                },
            ]),
            input_fields: None,
            interfaces: None,
            enum_values: None,
            possible_types: None,
        };
        assert!(is_relay(&relay));
    }
}
