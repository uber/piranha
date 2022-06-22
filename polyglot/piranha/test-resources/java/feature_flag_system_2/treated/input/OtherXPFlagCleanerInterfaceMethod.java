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


class XPFlagCleanerPositiveCases {

  private ExperimentInterface experimentation;

  private boolean ftBool = experimentation.isSomeFeature().getCachedValue();

  public void conditional_contains_stale_flag() {

    if (experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    }
  }

  public void conditional_with_else_contains_stale_flag() {

    if (experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool() {

    bool tBool = exp.isSomeFeature().getCachedValue();
    if (tBool && experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool(int a) {

    bool tBool = exp.isSomeFeature().getCachedValue();
    if (tBool && experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool(int a, bool abc) {

    bool tBool = exp.isSomeFeature().getCachedValue();
    if (!tBool && experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool_reassigned(int a, bool abc, int z) {
    // Currently if there is another assignment, variable will not be inlined.
    bool tBool = exp.isSomeFeature().getCachedValue();
    tBool = abc() && tBool;
    if (!tBool && experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool_reassigned_to_same_val(
      int a, bool abc, int z) {

    bool tBool = exp.isSomeFeature().getCachedValue();
    tBool = true;
    if (!tBool && experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_ftbool(int a) {

    if (ftBool && experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool_reassigned_ftbool(
      int a, bool abc, int z) {
    // Currently if there is another assignment, variable will not be inlined.
    ftBool = exp.isSomeFeature().getCachedValue();
    if (!ftBool && experimentation.isSomeFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }
}
