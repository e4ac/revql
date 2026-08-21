use crate::schema::{TypeMap, is_composite, unwrap_type_name};
use colored::{Color, Colorize};
use std::collections::HashSet;

const MAX_DEPTH: usize = 10;
const MAX_PATHS: usize = 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathStep {
    pub from_type: String,
    pub from_field: String,
}

impl PathStep {
    pub fn as_string(&self, color: bool) -> String {
        if color {
            format!("{}.{}", self.from_type.color(Color::Red), self.from_field.color(Color::Blue))
        } else {
            format!("{}.{}", self.from_type, self.from_field)
        }
    }
}

impl PathStep {
    pub fn new(from_type: impl Into<String>, from_field: impl Into<String>) -> Self {
        Self {
            from_type: from_type.into(),
            from_field: from_field.into(),
        }
    }
}

pub type Path = Vec<PathStep>;
pub type AllPaths = Vec<Path>;

pub fn find_all_paths(type_map: &TypeMap, start_type: &str, end_type: &str) -> AllPaths {
    let mut paths = vec![];
    let mut current_path = Path::default();
    let mut visited = HashSet::new();
    path_dfs(type_map, start_type, end_type, &mut current_path, &mut visited, &mut paths);
    paths
}

fn path_dfs(
    type_map: &TypeMap,
    current_type: &str,
    end_type: &str,
    current_path: &mut Path,
    visited: &mut HashSet<String>,
    paths: &mut Vec<Path>,
) {
    if paths.len() >= MAX_PATHS {
        return;
    }
    if current_type == end_type {
        paths.push(current_path.clone());
        return;
    }
    if current_path.len() >= MAX_DEPTH {
        return;
    }
    if !visited.insert(current_type.to_owned()) {
        return; // skip cycles
    }

    let Some(t) = type_map.get(current_type).filter(|t| is_composite(t)) else {
        visited.remove(current_type);
        return; // skip invalid types
    };

    let valid_fields = t.fields.iter().flatten().filter_map(|f| {
        let field_type = f.type_.as_ref()?;
        Some((f.name.as_ref()?, unwrap_type_name(&field_type.type_ref)?))
    });

    for (from_field, to_type) in valid_fields {
        let step = PathStep::new(current_type, from_field);
        current_path.push(step);
        path_dfs(type_map, to_type, end_type, current_path, visited, paths);
        current_path.pop();
    }
    visited.remove(current_type);
}

#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use super::*;
    use graphql_introspection_query::introspection_response::{
        __TypeKind, FullType, FullTypeFields, FullTypeFieldsType, TypeRef,
    };

    fn create_object_type(name: &str, fields: Vec<FullTypeFields>) -> FullType {
        FullType {
            kind: Some(__TypeKind::OBJECT),
            name: Some(name.to_owned()),
            description: None,
            fields: Some(fields),
            input_fields: None,
            interfaces: None,
            enum_values: None,
            possible_types: None,
        }
    }

    fn create_object_type_ref(name: &str) -> TypeRef {
        TypeRef {
            kind: Some(__TypeKind::OBJECT),
            name: Some(name.to_owned()),
            of_type: None,
        }
    }

    fn make_field(name: &str, type_name: &str) -> FullTypeFields {
        FullTypeFields {
            name: Some(name.to_owned()),
            description: None,
            args: None,
            type_: Some(FullTypeFieldsType {
                type_ref: create_object_type_ref(type_name),
            }),
            is_deprecated: None,
            deprecation_reason: None,
        }
    }

    #[test]
    fn test_find_all_paths_single_path() {
        let user_type = create_object_type("User", vec![make_field("profile", "Profile")]);
        let profile_type = create_object_type("Profile", vec![make_field("avatar", "Image")]);
        let image_type = create_object_type("Image", vec![]);
        let type_map = TypeMap::from([("User", &user_type), ("Profile", &profile_type), ("Image", &image_type)]);

        // User.profile -> Profile.avatar -> Image
        let paths = find_all_paths(&type_map, "User", "Image");
        assert_eq!(paths.len(), 1);
        assert!(paths.contains(&vec![PathStep::new("User", "profile"), PathStep::new("Profile", "avatar"),]));
    }

    #[test]
    fn test_find_all_paths_multiple_routes() {
        let user_type = create_object_type("User", vec![make_field("profile", "Profile"), make_field("posts", "Post")]);
        let profile_type = create_object_type("Profile", vec![make_field("avatar", "Image")]);
        let post_type = create_object_type("Post", vec![make_field("image", "Image")]);
        let image_type = create_object_type("Image", vec![]);
        let type_map = TypeMap::from([
            ("User", &user_type),
            ("Profile", &profile_type),
            ("Post", &post_type),
            ("Image", &image_type),
        ]);

        // User.profile -> Profile.avatar -> Image
        // User.posts -> Post.image -> Image
        let paths = find_all_paths(&type_map, "User", "Image");
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&vec![PathStep::new("User", "profile"), PathStep::new("Profile", "avatar"),]));
        assert!(paths.contains(&vec![PathStep::new("User", "posts"), PathStep::new("Post", "image"),]));
    }

    #[test]
    fn test_find_all_paths_diamond_path() {
        let user_fields = vec![make_field("profile", "Profile"), make_field("company", "Company")];
        let user_type = create_object_type("User", user_fields);
        let profile_type = create_object_type("Profile", vec![make_field("country", "Country")]);
        let company_type = create_object_type("Company", vec![make_field("country", "Country")]);
        let country_type = create_object_type("Country", vec![make_field("currency", "Currency")]);
        let currency_type = create_object_type("Currency", vec![]);
        let type_map = TypeMap::from([
            ("User", &user_type),
            ("Profile", &profile_type),
            ("Company", &company_type),
            ("Country", &country_type),
            ("Currency", &currency_type),
        ]);

        // User.profile -> Profile.country -> Country.currency -> Currency
        // User.company -> Company.country -> Country.currency -> Currency
        let paths = find_all_paths(&type_map, "User", "Currency");
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&vec![
            PathStep::new("User", "profile"),
            PathStep::new("Profile", "country"),
            PathStep::new("Country", "currency"),
        ]));
        assert!(paths.contains(&vec![
            PathStep::new("User", "company"),
            PathStep::new("Company", "country"),
            PathStep::new("Country", "currency"),
        ]));
    }

    #[test]
    fn test_find_all_paths_with_cycles() {
        let user_type = create_object_type("User", vec![make_field("posts", "Post")]);
        let post_type = create_object_type("Post", vec![make_field("author", "User"), make_field("image", "Image")]);
        let image_type = create_object_type("Image", vec![]);
        let type_map = TypeMap::from([("User", &user_type), ("Post", &post_type), ("Image", &image_type)]);

        // Post.image -> Image
        let paths = find_all_paths(&type_map, "User", "Image");
        assert_eq!(paths.len(), 1);
        assert!(paths.contains(&vec![PathStep::new("User", "posts"), PathStep::new("Post", "image"),]));
    }

    #[test]
    fn test_find_all_paths_nested_type() {
        let image_type = create_object_type("Image", vec![]);
        let image_type_ref = FullTypeFields {
            name: Some("avatar".to_owned()),
            description: None,
            args: None,
            type_: Some(FullTypeFieldsType {
                type_ref: TypeRef {
                    kind: Some(__TypeKind::NON_NULL),
                    name: None,
                    of_type: Some(Box::new(TypeRef {
                        kind: Some(__TypeKind::LIST),
                        name: None,
                        of_type: Some(Box::new(TypeRef {
                            kind: Some(__TypeKind::OBJECT),
                            name: Some("Image".to_owned()),
                            of_type: None,
                        })),
                    })),
                },
            }),
            is_deprecated: None,
            deprecation_reason: None,
        };
        let user_type = create_object_type("User", vec![make_field("profile", "Profile")]);
        let profile_type = create_object_type("Profile", vec![image_type_ref]);
        let type_map = TypeMap::from([("User", &user_type), ("Profile", &profile_type), ("Image", &image_type)]);

        // User.profile -> Profile.avatar -> Image
        let paths = find_all_paths(&type_map, "User", "Image");
        assert_eq!(paths.len(), 1);
        assert!(paths.contains(&vec![PathStep::new("User", "profile"), PathStep::new("Profile", "avatar"),]));
    }

    #[test]
    fn test_find_all_paths_instant_match() {
        let user_type = create_object_type("User", vec![]);
        let type_map = TypeMap::from([("User", &user_type)]);
        let paths = find_all_paths(&type_map, "User", "User");
        assert_eq!(paths.len(), 1);
        assert!(paths[0].is_empty());
    }

    #[test]
    fn test_find_all_paths_missing_types() {
        let user_type = create_object_type("User", vec![make_field("order", "Order")]);
        let type_map = TypeMap::from([("User", &user_type)]);
        let paths = find_all_paths(&type_map, "User", "Image");
        assert!(paths.is_empty());
    }

    #[test]
    fn test_find_all_paths_no_path() {
        let user_type = create_object_type("User", vec![]);
        let image_type = create_object_type("Image", vec![]);
        let type_map = TypeMap::from([("User", &user_type), ("Image", &image_type)]);
        let paths = find_all_paths(&type_map, "User", "Image");
        assert!(paths.is_empty());
    }
}
