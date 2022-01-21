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
import com.sun.source.tree.MemberSelectTree;
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

  /**
   * Returns a matcher for the argument passed to a new class creation.
   *
   * @param index of the argument to be matched
   * @param argMatcher The matcher for the argument
   * @return Returns true of the argument at index matches argMatcher, otherwise false.
   */
  public static Matcher<NewClassTree> newClassHasArgument(
      int index, Matcher<ExpressionTree> argMatcher) {
    return new NewClassArgument(index, argMatcher);
  }

  /**
   * Returns a matcher for the expression member select expresion
   *
   * @param matcher matcher for the member select expression
   * @return a matcher for the member select expression to be appled
   */
  public static Matcher<ExpressionTree> memberSelectExpression(Matcher<ExpressionTree> matcher) {
    return new MemberSelectExpression(matcher);
  }

  /**
   * Returns a matcher for the arguments passed to a new class creation.
   *
   * @param matchType - ALL, AT_LEAST_ONE, LAST
   * @param argMatcher The matcher for the argument
   * @return Returns true if ALL, AT_LEAST_ONE, or LAST arguments match, otherwise false.
   */
  public static Matcher<NewClassTree> newClassHasArgument(
      ChildMultiMatcher.MatchType matchType, Matcher<ExpressionTree> argMatcher) {
    return new NewClassHasArguments(matchType, argMatcher);
  }

  /**
   * Returns a matcher for a variable declaration statement.
   *
   * @param name of the variable
   * @param expressionTreeMatcher Matcher for the RHS
   * @return Returns true if the name and expressionTreeMatcher match a variable declaration
   *     statement
   */
  public static Matcher<VariableTree> variableNameInitializer(
      String name, Matcher<ExpressionTree> expressionTreeMatcher) {
    return (variableTree, state) -> {
      ExpressionTree initializer = variableTree.getInitializer();
      return variableTree.getName().toString().equals(name)
          && initializer != null
          && expressionTreeMatcher.matches(initializer, state);
    };
  }

  private static class MemberSelectExpression implements Matcher<ExpressionTree> {
    private Matcher<ExpressionTree> expressionTreeMatcher;

    MemberSelectExpression(Matcher<ExpressionTree> expressionTreeMatcher) {
      this.expressionTreeMatcher = expressionTreeMatcher;
    }

    @Override
    public boolean matches(ExpressionTree expressionTree, VisitorState state) {
      return expressionTree instanceof MemberSelectTree
          && expressionTreeMatcher.matches(
              ((MemberSelectTree) expressionTree).getExpression(), state);
    }
  }

  private static class NewClassArgument implements Matcher<NewClassTree> {
    private final int position;
    private final Matcher<ExpressionTree> argumentMatcher;

    NewClassArgument(int position, Matcher<ExpressionTree> argumentMatcher) {
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

  private static class NewClassHasArguments
      extends ChildMultiMatcher<NewClassTree, ExpressionTree> {

    NewClassHasArguments(MatchType matchType, Matcher<ExpressionTree> nodeMatcher) {
      super(matchType, nodeMatcher);
    }

    @Override
    protected Iterable<? extends ExpressionTree> getChildNodes(
        NewClassTree newClassTree, VisitorState state) {
      return newClassTree.getArguments();
    }
  }
}
