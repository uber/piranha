/*
 Copyright (c) 2022 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use handlebars::Handlebars;
use serde::Serialize;

pub struct JavaRules<'a> {
  register: Handlebars<'a>,
}

pub(crate) static MATCH_IDENTIFIER: &str = "FIND_IDENTIFIER";
pub(crate) static MATCH_ALL_PRIVATE_FIELD: &str = "MATCH_ALL_PRIVATE_FIELD";
pub(crate) static MATCH_PRIVATE_FIELD: &str = "MATCH_PRIVATE_FIELD";

impl JavaRules<'_> {
  pub fn new() -> Self {
    let mut register: Handlebars = Handlebars::new();

    register
      .register_template_string(
        MATCH_IDENTIFIER,
        "(
            (identifier) @identifier
            (#eq? @identifier \"{{name}}\")
        )",
      )
      .unwrap();

    register
      .register_template_string(
        MATCH_ALL_PRIVATE_FIELD,
        "(
            (field_declaration 
               (modifiers) @mod
               (variable_declarator name: (_)@name)) @field
            (#match? @mod \"private\")  
            )",
      )
      .unwrap();

    register
      .register_template_string(
        MATCH_PRIVATE_FIELD,
        "(
            (field_declaration 
               (modifiers) @mod
               (variable_declarator name: (_)@name)) @field
            (#match? @mod \"private\")
            (#match? @name \"{{name}}\")  
            )",
      )
      .unwrap();

    JavaRules { register }
  }

  pub fn render<T>(&self, name: &str, data: &T) -> String
  where
    T: Serialize,
  {
    self.register.render(name, &data).unwrap()
  }
}
