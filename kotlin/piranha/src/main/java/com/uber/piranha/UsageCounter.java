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

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableMap;
import com.google.errorprone.VisitorState;
import com.google.errorprone.util.ASTHelpers;
import com.sun.source.tree.IdentifierTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.Tree;
import com.sun.source.tree.VariableTree;
import com.sun.source.util.TreePath;
import com.sun.source.util.TreePathScanner;
import com.sun.source.util.TreeScanner;
import com.sun.tools.javac.code.Symbol;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;

/** Originally from com.uber.errorprone.checker.dagger.UsageCounter */
public final class UsageCounter {

  private UsageCounter() {
    /* Helper class only, not instantiable */
  }

  public static ImmutableMap<Symbol, CounterData> getUsageCounts(VisitorState state) {
    return getUsageCounts(state, state.getPath());
  }

  public static ImmutableMap<Symbol, CounterData> getUsageCounts(
      VisitorState state, TreePath path) {
    UsageCounter.CallScanner callScanner = new UsageCounter.CallScanner(state);
    callScanner.scan(path, null);
    ImmutableMap.Builder<Symbol, CounterData> builder = ImmutableMap.builder();
    for (VariableTree decl : callScanner.declaredInjectVars) {
      Symbol s = ASTHelpers.getSymbol(decl);
      CounterData counterData =
          new CounterData(
              DeclType.FIELD,
              decl,
              (callScanner.usedVars.containsKey(s) ? callScanner.usedVars.get(s) : 0));
      builder.put(s, counterData);
    }

    for (VariableTree decl : callScanner.declaredParamVars.keySet()) {
      Symbol s = ASTHelpers.getSymbol(decl);
      CounterData counterData =
          new CounterData(
              DeclType.PARAM,
              decl,
              (callScanner.usedVars.containsKey(s) ? callScanner.usedVars.get(s) : 0));
      builder.put(s, counterData);
    }
    return builder.build();
  }

  public static ImmutableMap<Symbol, Integer> getRawUsageCounts(Tree tree) {
    RawUsageCountsScanner scanner = new RawUsageCountsScanner();
    scanner.scan(tree, null);
    return ImmutableMap.copyOf(scanner.usedVars);
  }

  private static void addUse(Map<Symbol, Integer> usedVars, Symbol symbol) {
    if (usedVars.containsKey(symbol)) {
      usedVars.put(symbol, usedVars.get(symbol) + 1);
    } else {
      usedVars.put(symbol, 1);
    }
  }

  static class CallScanner extends TreePathScanner<Void, Void> {
    final Set<VariableTree> declaredInjectVars = new LinkedHashSet<>();
    final HashMap<VariableTree, Symbol.MethodSymbol> declaredParamVars =
        new HashMap<VariableTree, Symbol.MethodSymbol>();
    final Map<Symbol, Integer> usedVars = new LinkedHashMap<>();
    final VisitorState state;

    CallScanner(VisitorState state) {
      this.state = state;
    }

    @Override
    public Void visitMemberSelect(MemberSelectTree tree, Void unused) {
      addUse(usedVars, ASTHelpers.getSymbol(tree));
      return super.visitMemberSelect(tree, null);
    }

    @Override
    public Void visitIdentifier(IdentifierTree tree, Void unused) {
      addUse(usedVars, ASTHelpers.getSymbol(tree));
      return super.visitIdentifier(tree, null);
    }

    @Override
    public Void visitMethod(MethodTree tree, Void unused) {
      Symbol.MethodSymbol mSym = ASTHelpers.getSymbol(tree);
      if (ASTHelpers.hasAnnotation(mSym, "dagger.Provides", state)) {
        for (VariableTree vt : tree.getParameters()) {
          declaredParamVars.put(vt, mSym);
        }
      }
      return super.visitMethod(tree, null);
    }

    @Override
    public Void visitVariable(VariableTree tree, Void unused) {
      Symbol.VarSymbol vSym = ASTHelpers.getSymbol(tree);
      if (ASTHelpers.hasAnnotation(vSym, "javax.inject.Inject", state)) {
        declaredInjectVars.add(tree);
      }
      return super.visitVariable(tree, null);
    }
  }

  static class RawUsageCountsScanner extends TreeScanner<Void, Void> {
    final Map<Symbol, Integer> usedVars = new LinkedHashMap<>();

    @Override
    public Void visitMemberSelect(MemberSelectTree tree, Void unused) {
      addUse(usedVars, ASTHelpers.getSymbol(tree));
      return super.visitMemberSelect(tree, null);
    }

    @Override
    public Void visitIdentifier(IdentifierTree tree, Void unused) {
      addUse(usedVars, ASTHelpers.getSymbol(tree));
      return super.visitIdentifier(tree, null);
    }
  }

  public enum DeclType {
    FIELD,
    PARAM,
  }

  public static class CounterData {
    public final DeclType declType;
    public final VariableTree declaration;
    public final int count;

    public CounterData(DeclType declType, VariableTree declaration, int count) {
      Preconditions.checkArgument(count >= 0, "Count must be positive.");
      this.declType = declType;
      this.declaration = declaration;
      this.count = count;
    }
  }
}
