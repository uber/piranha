[PiranhaOutputSummary { path: "test-resources/java/insert_field_and_import/input/TestClass.java", original_content: "/**\n * Copyright (c) 2023 Uber Technologies, Inc.\n *\n * <p>Licensed under the Apache License, Version 2.0 (the \"License\"); you may not use this file\n * except in compliance with the License. You may obtain a copy of the License at\n *\n * <p>http://www.apache.org/licenses/LICENSE-2.0\n *\n * <p>Unless required by applicable law or agreed to in writing, software distributed under the\n * License is distributed on an \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either\n * express or implied. See the License for the specific language governing permissions and\n * limitations under the License.\n */\npackage com.uber.piranha;\n\n\nclass FooBar {\n\n  private String address;\n\n  private String phoneNumber;\n\n  private String prefix;\n\n  private String street;\n\n  private String city;\n\n  private String country;\n\n  public String getAddress() {\n    return address;\n  }\n\n  public String getPhoneNumber() {\n    return phoneNumber;\n  }\n\n  public String getStreet() {\n    return street;\n  }\n\n  public String getPrefix() {\n    return prefix;\n  }\n\n  public FooBar(String address) {\n    this.address = address;\n  }\n}\n", content: "/**\n * Copyright (c) 2023 Uber Technologies, Inc.\n *\n * <p>Licensed under the Apache License, Version 2.0 (the \"License\"); you may not use this file\n * except in compliance with the License. You may obtain a copy of the License at\n *\n * <p>http://www.apache.org/licenses/LICENSE-2.0\n *\n * <p>Unless required by applicable law or agreed to in writing, software distributed under the\n * License is distributed on an \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either\n * express or implied. See the License for the specific language governing permissions and\n * limitations under the License.\n */\npackage com.uber.piranha;\nimport java.util.List;\n\n\n\nclass FooBar {\n  private List<String> names;\n private String address;\nprivate String phoneNumber;\nprivate String prefix;\nprivate String street;\nprivate String city;\nprivate String country;\npublic String getAddress() {\n    return address;\n  }\npublic String getPhoneNumber() {\n    return phoneNumber;\n  }\npublic String getStreet() {\n    return street;\n  }\npublic String getPrefix() {\n    return prefix;\n  }\npublic FooBar(String address) {\n    this.address = address;\n  }\n}\n", matches: [], rewrites: [Edit { p_match: Match { matched_string: "{\n\n  private String address;\n\n  private String phoneNumber;\n\n  private String prefix;\n\n  private String street;\n\n  private String city;\n\n  private String country;\n\n  public String getAddress() {\n    return address;\n  }\n\n  public String getPhoneNumber() {\n    return phoneNumber;\n  }\n\n  public String getStreet() {\n    return street;\n  }\n\n  public String getPrefix() {\n    return prefix;\n  }\n\n  public FooBar(String address) {\n    this.address = address;\n  }\n}", range: Range { start_byte: 652, end_byte: 1111, start_point: Point { row: 16, column: 13 }, end_point: Point { row: 49, column: 1 } }, matches: {"class_body": "{\n\n  private String address;\n\n  private String phoneNumber;\n\n  private String prefix;\n\n  private String street;\n\n  private String city;\n\n  private String country;\n\n  public String getAddress() {\n    return address;\n  }\n\n  public String getPhoneNumber() {\n    return phoneNumber;\n  }\n\n  public String getStreet() {\n    return street;\n  }\n\n  public String getPrefix() {\n    return prefix;\n  }\n\n  public FooBar(String address) {\n    this.address = address;\n  }\n}", "class_name": "FooBar", "class_declaration": "class FooBar {\n\n  private String address;\n\n  private String phoneNumber;\n\n  private String prefix;\n\n  private String street;\n\n  private String city;\n\n  private String country;\n\n  public String getAddress() {\n    return address;\n  }\n\n  public String getPhoneNumber() {\n    return phoneNumber;\n  }\n\n  public String getStreet() {\n    return street;\n  }\n\n  public String getPrefix() {\n    return prefix;\n  }\n\n  public FooBar(String address) {\n    this.address = address;\n  }\n}", "class_members": "private String address;\nprivate String phoneNumber;\nprivate String prefix;\nprivate String street;\nprivate String city;\nprivate String country;\npublic String getAddress() {\n    return address;\n  }\npublic String getPhoneNumber() {\n    return phoneNumber;\n  }\npublic String getStreet() {\n    return street;\n  }\npublic String getPrefix() {\n    return prefix;\n  }\npublic FooBar(String address) {\n    this.address = address;\n  }"}, associated_comma: None, associated_comments: [] }, replacement_string: "{\n  private List<String> names;\n private String address;\nprivate String phoneNumber;\nprivate String prefix;\nprivate String street;\nprivate String city;\nprivate String country;\npublic String getAddress() {\n    return address;\n  }\npublic String getPhoneNumber() {\n    return phoneNumber;\n  }\npublic String getStreet() {\n    return street;\n  }\npublic String getPrefix() {\n    return prefix;\n  }\npublic FooBar(String address) {\n    this.address = address;\n  }\n}", matched_rule: "add_field_declaration" }, Edit { p_match: Match { matched_string: "package com.uber.piranha;", range: Range { start_byte: 611, end_byte: 636, start_point: Point { row: 13, column: 0 }, end_point: Point { row: 13, column: 25 } }, matches: {"pkg_dcl": "package com.uber.piranha;"}, associated_comma: None, associated_comments: [] }, replacement_string: "package com.uber.piranha;\nimport java.util.List;\n", matched_rule: "add_import" }] }]