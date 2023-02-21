/**
 * Copyright (c) 2022 Uber Technologies, Inc.
 *
 * <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 * <p>http://www.apache.org/licenses/LICENSE-2.0
 *
 * <p>Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.uber.piranha;

import java.util.List;

class FooBar {

  private List<String> names;

  private String address;

  private String phoneNumber;

  private String prefix;

  private String street;

  private String city;

  private String country;

  public String getAddress() {
    return address;
  }

  public String getPhoneNumber() {
    return phoneNumber;
  }

  public String getStreet() {
    return street;
  }

  public String getPrefix() {
    return prefix;
  }

  public FooBar(String address) {
    this.address = address;
  }
}
