/**
 * Copyright (c) 2019 Uber Technologies, Inc.
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

import dagger.Provides;
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import javax.inject.Inject;

class XPFlagCleanerPositiveCases {

  private ExperimentInterface experimentation;

  private boolean ftBool = experimentation.isStaleFeature().getCachedValue();

  public void conditional_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isStaleFeature().getCachedValue()) {
      System.out.println("Hello World");
    }
  }

  public void conditional_with_else_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isStaleFeature().getCachedValue()) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    bool tBool = exp.isStaleFeature().getCachedValue();
    if (tBool && true) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool(int a) {
    // BUG: Diagnostic contains: Cleans stale XP flags
    bool tBool = exp.isStaleFeature().getCachedValue();
    if (tBool && true) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool(int a, bool abc) {
    // BUG: Diagnostic contains: Cleans stale XP flags
    bool tBool = exp.isStaleFeature().getCachedValue();
    if (!tBool && true) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool_reassigned(int a, bool abc, int z) {
    // Currently if there is another assignment, variable will not be inlined.
    bool tBool = exp.isStaleFeature().getCachedValue();
    tBool = abc() && tBool;
    if (!tBool && true) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool_reassigned_to_same_val(int a, bool abc, int z) {
    // BUG: Diagnostic contains: Cleans stale XP flags
    bool tBool = exp.isStaleFeature().getCachedValue();
    tBool = true;
    if (!tBool && true) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_ftbool(int a) {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (ftBool && true) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void conditional_with_else_contains_stale_flag_tbool_reassigned_ftbool(int a, bool abc, int z) {
    // Currently if there is another assignment, variable will not be inlined.
    ftBool = exp.isStaleFeature().getCachedValue();
    if (!ftBool && true) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }  
}