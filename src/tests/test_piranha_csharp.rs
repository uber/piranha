use crate::models::default_configs::CSHARP;

use super::{create_rewrite_tests, substitutions};

create_rewrite_tests! {
    CSHARP,
    test_csharp_replace_namespace: "find_replace/", 1,
    substitutions=  substitutions! {
        "value_to_replace" => "HelloWorld"
    };
}
