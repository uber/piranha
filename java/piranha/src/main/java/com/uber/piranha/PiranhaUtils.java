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

import com.google.errorprone.VisitorState;
import com.google.errorprone.matchers.ChildMultiMatcher;
import com.google.errorprone.matchers.Matcher;
import com.google.errorprone.util.ASTHelpers;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.NewClassTree;
import com.sun.source.tree.VariableTree;
import com.sun.source.util.TreePath;

public class PiranhaUtils {
  public static final String DELETE_REQUEST_COMMENT =
      "//[PIRANHA_DELETE_FILE_SEQ] Delete this class.\n";

  public static final String HELPER_CLASS = "// [PIRANHA_STALE_PLUGIN_HELPER_CLASS]";

  public static String expressionToSimpleName(ExpressionTree tree) {
    return ASTHelpers.getSymbol(tree).getSimpleName().toString();
  }

  public static boolean isUnitTestMethod(MethodTree tree, VisitorState state) {
    // A simple heuristic, but useful for now. Consider supporting other testing frameworks in the
    // future.
    return ASTHelpers.hasAnnotation(tree, "org.junit.Test", state);
  }

  public static boolean isPrefixPath(TreePath prefix, TreePath path) {
    while (path != null) {
      if (path.equals(prefix)) {
        return true;
      }
      path = path.getParentPath();
    }
    return false;
  }

  public static class NewClassArgument implements Matcher<NewClassTree> {
    private final int position;
    private final Matcher<ExpressionTree> argumentMatcher;

    public NewClassArgument(int position, Matcher<ExpressionTree> argumentMatcher) {
      this.position = position;
      this.argumentMatcher = argumentMatcher;
    }

    @Override
    public boolean matches(NewClassTree newClassTree, VisitorState state) {
      if (newClassTree.getArguments().size() <= position) {
        return false;
      }
      return argumentMatcher.matches(newClassTree.getArguments().get(position), state);
    }
  }

  public static class NewClassHasArguments extends ChildMultiMatcher<NewClassTree, ExpressionTree> {

    public NewClassHasArguments(MatchType matchType, Matcher<ExpressionTree> nodeMatcher) {
      super(matchType, nodeMatcher);
    }

    @Override
    protected Iterable<? extends ExpressionTree> getChildNodes(
        NewClassTree newClassTree, VisitorState state) {
      return newClassTree.getArguments();
    }
  }

  public static Matcher<VariableTree> variableNameInitializer(
      String name, Matcher<ExpressionTree> expressionTreeMatcher) {
    return (variableTree, state) -> {
      ExpressionTree initializer = variableTree.getInitializer();
      return variableTree.getName().toString().equals(name)
          && initializer != null
          && expressionTreeMatcher.matches(initializer, state);
    };
  }
}
