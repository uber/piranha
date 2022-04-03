/*
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

import static com.google.errorprone.BugPattern.SeverityLevel.SUGGESTION;
import static com.google.errorprone.matchers.ChildMultiMatcher.MatchType.AT_LEAST_ONE;
import static com.google.errorprone.matchers.Matchers.anyOf;
import static com.google.errorprone.matchers.Matchers.argument;
import static com.google.errorprone.matchers.Matchers.contains;
import static com.google.errorprone.matchers.Matchers.instanceMethod;
import static com.google.errorprone.matchers.Matchers.isSameType;
import static com.google.errorprone.matchers.Matchers.receiverOfInvocation;
import static com.google.errorprone.matchers.Matchers.staticMethod;
import static com.google.errorprone.matchers.Matchers.symbolMatcher;
import static com.google.errorprone.matchers.Matchers.toType;
import static com.uber.piranha.PiranhaUtils.DELETE_REQUEST_COMMENT;
import static com.uber.piranha.PiranhaUtils.memberSelectExpression;
import static com.uber.piranha.PiranhaUtils.newClassHasArgument;

import com.facebook.infer.annotation.Initializer;
import com.google.auto.service.AutoService;
import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableCollection;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import com.google.errorprone.BugPattern;
import com.google.errorprone.ErrorProneFlags;
import com.google.errorprone.VisitorState;
import com.google.errorprone.bugpatterns.BugChecker;
import com.google.errorprone.fixes.SuggestedFix;
import com.google.errorprone.matchers.Description;
import com.google.errorprone.matchers.Matcher;
import com.google.errorprone.matchers.Matchers;
import com.google.errorprone.util.ASTHelpers;
import com.google.errorprone.util.FindIdentifiers;
import com.sun.source.tree.AnnotationTree;
import com.sun.source.tree.AssignmentTree;
import com.sun.source.tree.BinaryTree;
import com.sun.source.tree.BlockTree;
import com.sun.source.tree.ClassTree;
import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.ConditionalExpressionTree;
import com.sun.source.tree.ExpressionStatementTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.IfTree;
import com.sun.source.tree.ImportTree;
import com.sun.source.tree.LiteralTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodInvocationTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.NewClassTree;
import com.sun.source.tree.ParenthesizedTree;
import com.sun.source.tree.ReturnTree;
import com.sun.source.tree.StatementTree;
import com.sun.source.tree.Tree;
import com.sun.source.tree.Tree.Kind;
import com.sun.source.tree.UnaryTree;
import com.sun.source.tree.VariableTree;
import com.sun.source.util.TreePath;
import com.sun.tools.javac.code.Symbol;
import com.sun.tools.javac.tree.JCTree;
import com.uber.piranha.config.Config;
import com.uber.piranha.config.MethodRecord;
import com.uber.piranha.config.PiranhaConfigurationException;
import com.uber.piranha.config.PiranhaEnumRecord;
import com.uber.piranha.config.PiranhaMethodRecord;
import com.uber.piranha.testannotations.AnnotationArgument;
import com.uber.piranha.testannotations.ResolvedTestAnnotation;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import java.util.stream.Stream;
import javax.annotation.Nullable;
import javax.lang.model.element.ElementKind;

/**
 * This is the core PiranhaJava checker and code rewriting class.
 *
 * <p>This checker iterates over the AST of each compilation unit, performing the following steps:
 * a) Replaces occurrences of flag checking APIs with boolean constants based on Piranha's
 * configuration and flag name arguments, and removes outdated flag setting operations. b)
 * Simplifies boolean expressions including these new constants. c) Deletes code that has become
 * unreachable due to conditional guards which can no longer evaluate to true. d) Deletes outdated
 * enums and classes representing the removed flags and/or their treatment conditions.
 *
 * <p>In some cases, Piranha/XPFlagCleaner will want to delete a whole file. Since EP has no
 * facilities for creating a patch that removes an entire file, it will instead replace its contents
 * by an empty enum/class and add a special comment indicating it must be removed. Automated scripts
 * using this code can then perform the actual file deletion previous to creating a PR or diff.
 *
 * @author murali@uber.com (Murali Krishna Ramanathan)
 */
@AutoService(BugChecker.class)
@BugPattern(
    name = "Piranha",
    altNames = {"XPFlagCleaner"},
    summary = "Cleans stale XP flags.",
    severity = SUGGESTION)
public class XPFlagCleaner extends BugChecker
    implements BugChecker.AssignmentTreeMatcher,
        BugChecker.BinaryTreeMatcher,
        BugChecker.CompilationUnitTreeMatcher,
        BugChecker.ConditionalExpressionTreeMatcher,
        BugChecker.ClassTreeMatcher,
        BugChecker.ExpressionStatementTreeMatcher,
        BugChecker.IfTreeMatcher,
        BugChecker.ImportTreeMatcher,
        BugChecker.MethodInvocationTreeMatcher,
        BugChecker.ReturnTreeMatcher,
        BugChecker.UnaryTreeMatcher,
        BugChecker.VariableTreeMatcher,
        BugChecker.MethodTreeMatcher {

  /**
   * Do not try to auto-delete imports with these common/generic names, as multiple treatment groups
   * for different flags are likely to re-use these names.
   */
  private static final ImmutableSet<String> COMMON_GROUP_NAMES =
      ImmutableSet.of("control", "enabled", "disabled", "treatment", "treated");

  private static final int DONTCARE = -1;
  private static final String TRUE = "true";
  private static final String FALSE = "false";
  private static final String EMPTY = "";

  /**
   * Used to defer initialization until after traversal has begun, since Error Prone eats all error
   * messages thrown inside the constructor.
   */
  private boolean initialized = false;

  /**
   * When no configuration whatsoever is provided, and -Xep:Piranha:DisabledUnlessConfigured=true,
   * disable the checker silently. This is needed for some large multi-target builds which load the
   * EP plug-in for a subset of targets (which makes a global `-Xep:Piranha:OFF` cause an error),
   * but then pass the configuration to run it on a smaller subset (which will cause a
   * PiranhaConfigurationException to be thrown for those targets in the set difference).
   */
  private boolean disabled = false;

  @Nullable private ErrorProneFlags flags = null;

  private String xpFlagName = "_xpflag_dummy";

  /** Enumerator for different values returned by expression evaluation */
  private enum Value {
    TRUE,
    FALSE,
    BOT
  }

  /** Identify the appropriate API for experiments */
  public enum API {
    IS_TREATED,
    IS_CONTROL,
    IS_TREATMENT_GROUP_CHECK,
    SET_TREATED,
    SET_CONTROL,
    DELETE_METHOD,
    UNKNOWN
  }

  @Nullable private Symbol xpSym = null;
  private boolean isTreated = true;
  private String treatmentGroup = "";

  @Nullable
  private String treatmentGroupsEnum = null; // FQN of the enum containing the treatment group names

  /** Caches results for enums with matching constructor args */
  private final Map<EnumWithClassSymbol, Boolean> enumsMatchingConstructorArgsCache =
      new HashMap<>();

  /**
   * when source is refactored, this specifies the end position for the refactoring until the
   * processing of the source reaches this position. otherwise, the value is -1.
   */
  private int endPos = DONTCARE;

  /**
   * Information provided in the properties.json config file.
   *
   * <p>Can't be final due to init() method, but should not be assigned anywhere else
   */
  private Config config;

  /** State used to track usage counts and delete corresponding declarations if needed. */
  @Nullable private TreePath cuPath = null;

  private boolean countsCollected = false;
  @Nullable private ImmutableMap<Symbol, UsageCounter.CounterData> usageCounts = null;
  @Nullable private Map<Symbol, Integer> deletedUsages = null;

  // Set to skip refactoring under a given path (e.g. because we have deleted the entire AST under
  // that path)
  @Nullable private TreePath skipWithinPath = null;

  /**
   * Copied from NullAway comment. Error Prone requires us to have an empty constructor for each
   * Plugin, in addition to the constructor taking an ErrorProneFlags object. This constructor
   * should not be used anywhere else.
   */
  public XPFlagCleaner() {}

  public XPFlagCleaner(ErrorProneFlags flags) {
    this.flags = flags;
  }

  @SuppressWarnings("unchecked") // Needed for JSON parsing.
  @Initializer
  void init(ErrorProneFlags flags) throws PiranhaConfigurationException {
    Optional<String> s = flags.get("Piranha:FlagName");
    Optional<String> f = flags.get("Piranha:Config");
    if (!s.isPresent()
        && !f.isPresent()
        && flags.getBoolean("Piranha:DisabledUnlessConfigured").orElse(false)
        && !flags.getBoolean("Piranha:IsTreated").isPresent()
        && !flags.getBoolean("Piranha:TreatmentGroup").isPresent()
        && this.defaultSeverity().equals(SUGGESTION)) {
      // No configuration present at all, disable Piranha checker
      disabled = true;
      this.config = Config.emptyConfig();
      return;
    }

    if (s.isPresent()) {
      if (!EMPTY.equals(s.get().trim())) xpFlagName = s.get();
      isTreated = flags.getBoolean("Piranha:IsTreated").orElse(true);
      treatmentGroup = flags.get("Piranha:TreatmentGroup").orElse("").toLowerCase();
    } else {
      throw new PiranhaConfigurationException("Piranha:FlagName is missing");
    }

    if (f.isPresent()) {
      String configFile = f.get();
      Optional<String> argumentIndexOptional = flags.get("Piranha:ArgumentIndexOptional");
      boolean isArgumentIndexOptional = false;
      if (argumentIndexOptional.isPresent() && TRUE.equalsIgnoreCase(argumentIndexOptional.get())) {
        isArgumentIndexOptional = true;
      }
      this.config = Config.fromJSONFile(configFile, isArgumentIndexOptional);
    } else {
      throw new PiranhaConfigurationException("Piranha:Config is missing");
    }
    initialized = true;
  }

  private boolean shouldSkip(VisitorState state) {
    return disabled
        || (skipWithinPath != null && PiranhaUtils.isPrefixPath(skipWithinPath, state.getPath()));
  }

  // When deleting a large block of code, skip scanning statements/expressions within that path
  private void skipWithin(TreePath path) {
    skipWithinPath = path;
  }

  // We call this lazily only when needed, meaning when a symbol usage is first deleted for this
  // Compilation Unit.
  private void computeSymbolCounts(VisitorState visitorState) {
    Preconditions.checkNotNull(cuPath, "Compilation Unit TreePath should be set by this point.");
    Preconditions.checkArgument(
        !countsCollected, "This shouldn't be called more than once per Compilation Unit");
    deletedUsages = new LinkedHashMap<>();
    // We count all usages stating at the root of the current compilation unit.
    usageCounts = UsageCounter.getUsageCounts(visitorState, cuPath);
    countsCollected = true;
  }

  private void decrementSymbolUsage(
      Symbol symbol, VisitorState visitorState, SuggestedFix.Builder builder) {
    // Run UsageCounter and check if this is a variable of interest as per that class.
    if (!countsCollected) {
      computeSymbolCounts(visitorState);
    }
    Preconditions.checkNotNull(usageCounts, "The code above should set usage counts info");
    Preconditions.checkNotNull(deletedUsages, "The code above should set deleted usages info");
    if (!usageCounts.containsKey(symbol)) {
      // Not a variable tracked by UsageCounter or UsageCheckers
      return;
    }
    // Then, update the number of deletions of `symbol`
    int perSymbolDeletedUsages = 1;
    if (deletedUsages.containsKey(symbol)) {
      perSymbolDeletedUsages += deletedUsages.get(symbol);
    }
    deletedUsages.put(symbol, perSymbolDeletedUsages);
    // Finally, check if this number of deletions equals the entire usage count and patch
    // accordingly.
    UsageCounter.CounterData counterData = usageCounts.get(symbol);
    Preconditions.checkArgument(counterData.count >= perSymbolDeletedUsages);
    if (counterData.count == perSymbolDeletedUsages) {
      // Remove the variable declaration.
      builder.delete(counterData.declaration);
    }
  }

  private void decrementAllSymbolUsages(
      Tree tree, VisitorState visitorState, SuggestedFix.Builder builder) {
    Map<Symbol, Integer> deletedUsages = UsageCounter.getRawUsageCounts(tree);
    for (Symbol s : deletedUsages.keySet()) {
      decrementSymbolUsage(s, visitorState, builder);
    }
  }

  @Override
  public Description matchCompilationUnit(
      CompilationUnitTree compilationUnitTree, VisitorState visitorState) {
    if (!initialized) {
      Preconditions.checkNotNull(
          flags,
          "The configuration-aware constructor should have been called at this point, and flags set to "
              + "a non-null value.");
      init(flags);
    }
    if (disabled) return Description.NO_MATCH;
    endPos = DONTCARE; // Important: reset end position for calculating overlaps between files/CUs
    if (countsCollected) {
      // Clear out this info
      countsCollected = false;
      usageCounts = null;
      deletedUsages = null;
    }
    cuPath = visitorState.getPath();
    return Description.NO_MATCH;
  }

  @Override
  public String linkUrl() {
    return this.config.getLinkURL();
  }

  /* Returns the appropriate XP API, if any, as given by the expression */
  private API getXPAPI(ExpressionTree et, VisitorState state) {
    et = ASTHelpers.stripParentheses(et);
    Kind k = et.getKind();
    if (k.equals(Tree.Kind.METHOD_INVOCATION)) {
      MethodInvocationTree mit = (MethodInvocationTree) et;
      if (!mit.getMethodSelect().getKind().equals(Kind.MEMBER_SELECT)) {
        return API.UNKNOWN;
      }
      ImmutableCollection<PiranhaMethodRecord> methodRecords =
          this.config.getMethodRecordsForName(mit, state);
      if (methodRecords.size() > 0) {
        return getXPAPI(mit, state, methodRecords);
      }
    }
    return API.UNKNOWN;
  }

  /**
   * This method assumes that the method record's name and the method invocation tree's name match.
   * The method reports a match if the receiver, relevant the argument index and the return type of
   * the method record match to that of the method invocation tree. Note that these fields are
   * optional.
   *
   * @param methodRecord candidate method record to match against
   * @param state visitor state
   * @param mit method invocation tree
   * @return true if method record matches tree, otherwise false.
   */
  private boolean methodRecordMatcher(
      MethodRecord methodRecord, VisitorState state, MethodInvocationTree mit) {
    // Method's receiver must match record's receiver type (if any)
    boolean receiverTypeMatches =
        !methodRecord.getReceiverType().isPresent()
            || receiverOfInvocation(
                    (receiver, st) ->
                        isSameType(methodRecord.getReceiverType().get()).matches(receiver, st))
                .matches(mit, state);
    if (!receiverTypeMatches) {
      return false;
    }

    // Method must have flag at argument index specified by record (if any)
    boolean argumentMatchesFlagName =
        !methodRecord.getArgumentIdx().isPresent()
            || argument(
                    methodRecord.getArgumentIdx().get(),
                    (arg, st) ->
                        isArgumentMatchesFlagName(arg, state) || isArgumentMatchesFlagMethod(arg))
                .matches(mit, state);
    if (!argumentMatchesFlagName) {
      return false;
    }

    // Method's return must match record's return type (if any)
    boolean returnTypeMatches =
        !methodRecord.getReturnType().isPresent()
            || isSameType(methodRecord.getReturnType().get()).matches(mit, state);
    return returnTypeMatches;
  }

  private Matcher<ExpressionTree> enumConstructorArgsContainsFlagNameMatcher(
      ImmutableCollection<PiranhaEnumRecord> enumRecords) {
    Matcher<ExpressionTree> matchFlag =
        (arg, state) ->
            arg.getKind() == Kind.STRING_LITERAL
                && ((LiteralTree) arg).getValue().equals(xpFlagName);

    return toType(
        NewClassTree.class,
        (nct, st) -> {
          Predicate<PiranhaEnumRecord> matchPiranhaEnumRecord =
              enumRecord ->
                  enumRecord
                      .getArgumentIdx()
                      .map(index -> newClassHasArgument(index, matchFlag).matches(nct, st))
                      .orElseGet(
                          () -> newClassHasArgument(AT_LEAST_ONE, matchFlag).matches(nct, st));
          return enumRecords.stream().anyMatch(matchPiranhaEnumRecord);
        });
  }

  private API getXPAPI(
      MethodInvocationTree mit,
      VisitorState state,
      ImmutableCollection<PiranhaMethodRecord> methodRecordsForName) {
    for (PiranhaMethodRecord methodRecord : methodRecordsForName) {
      if (methodRecordMatcher(methodRecord, state, mit)) {
        return methodRecord.getApiType();
      }
    }
    return API.UNKNOWN;
  }

  // If the cleanup option 'flag_method_name' is provided, this method will check
  // if the argument is a method invocation matches the provided flag method name.
  private boolean isArgumentMatchesFlagMethod(ExpressionTree arg) {
    return arg instanceof MethodInvocationTree
        && config
            .getFlagMethodName()
            .map(
                flagMethodName ->
                    config.getMethodName((MethodInvocationTree) arg).equals(flagMethodName))
            .orElse(false);
  }

  private boolean isArgumentMatchesFlagName(ExpressionTree arg, VisitorState state) {
    Symbol argSym = ASTHelpers.getSymbol(arg);
    return isLiteralTreeAndMatchesFlagName(arg)
        || isVarSymbolAndMatchesFlagName(argSym)
        || isSymbolAndMatchesFlagName(argSym)
        || (this.config.hasEnumRecords() && isMatchingEnumFieldValue(arg, state));
  }

  /*
   * Matches on an enum in an expression tree. Covers the following 4 cases:
   * 1. Unqualified import enum constant (e.g. "STALE_FLAG")
   * 2. Qualified enum name (e.g. "TestExperimentName.STALE_FLAG")
   * 3. Unqualified enum constant with method call (e.g. "STALE_FLAG.getKey()")
   * 4. Qualified enum name with method call (e.g. "TestExperimentName.STALE_FLAG.getKey()")
   */
  private boolean isMatchingEnumFieldValue(ExpressionTree tree, VisitorState state) {
    // Handles the first two scenarios.
    Matcher<ExpressionTree> enumFieldValueMatcher =
        symbolMatcher(
            (sym, st) ->
                isEnumConstantMatchingFlagName(
                    sym.getSimpleName().toString(), ASTHelpers.enclosingClass(sym), state));
    // Handles the last two scenarios.
    return anyOf(
            enumFieldValueMatcher,
            Matchers.methodInvocation(
                anyOf(enumFieldValueMatcher, memberSelectExpression(enumFieldValueMatcher))))
        .matches(tree, state);
  }

  /**
   * Finds whether the current enum constant contains the flag string in its constructor arguments.
   * Must also have a valid configuration record corresponding to this enum in order to return true.
   *
   * @param enumName Unqualified enum constant name (e.g. STALE_FLAG)
   * @param classSymbol Enum class symbol for the corresponding constant
   * @param state Used to find enum class in AST
   */
  public boolean isEnumConstantMatchingFlagName(
      String enumName, @Nullable Symbol.ClassSymbol classSymbol, VisitorState state) {
    if (classSymbol == null) return false;
    // Skip any enums that have no configuration
    ImmutableCollection<PiranhaEnumRecord> enumRecords =
        this.config.getEnumRecordsForName(classSymbol.getSimpleName().toString());
    if (enumRecords.isEmpty()) {
      return false;
    }

    // Check cached matches
    EnumWithClassSymbol enumWithClassSymbol = new EnumWithClassSymbol(enumName, classSymbol);
    if (enumsMatchingConstructorArgsCache.containsKey(enumWithClassSymbol)) {
      return enumsMatchingConstructorArgsCache.get(enumWithClassSymbol);
    }

    // TODO: Find constructor arguments purely through the symbol table; without the need for state
    //       Then, we will be able remove the following exception
    ClassTree enumClassTree = ASTHelpers.findClass(classSymbol, state);
    if (enumClassTree == null) {
      throw new PiranhaRuntimeException(
          "Detected enum constant of class "
              + classSymbol.className()
              + ", which is mentioned by"
              + " Piranha's configuration as part of enumProperties. However, enum definition source"
              + " is not available when looking at this usage (this can happen when cleaning up flags"
              + " across build targets)."
              + "\n\nIf you are trying to use Piranha to clean up enum flags based on the string"
              + " argument to their constructor (e.g. using enumProperties), the current"
              + " implementation will fail to match flag usages on a separate target as that"
              + " containing the enum source, resulting in partial clean up.");
    }

    Matcher<Tree> matchVarDeclWithNewClassInitPassingFlag =
        contains(
            VariableTree.class,
            PiranhaUtils.variableNameInitializer(
                enumName, enumConstructorArgsContainsFlagNameMatcher(enumRecords)));

    boolean result = matchVarDeclWithNewClassInitPassingFlag.matches(enumClassTree, state);

    enumsMatchingConstructorArgsCache.put(enumWithClassSymbol, result);
    return result;
  }

  /**
   * Checks for {@link Symbol} and the flag name
   *
   * @return True if matches. Otherwise false
   */
  private boolean isSymbolAndMatchesFlagName(Symbol argSym) {
    return argSym != null && (argSym.equals(xpSym) || argSym.toString().equals(xpFlagName));
  }
  /**
   * Checks for {@link com.sun.tools.javac.code.Symbol.VarSymbol} and the flag name
   *
   * @return True if matches. Otherwise false
   * @param argSym : symbol of the argument
   */
  private boolean isVarSymbolAndMatchesFlagName(Symbol argSym) {
    if (argSym instanceof Symbol.VarSymbol) {
      Object constantValue = ((Symbol.VarSymbol) argSym).getConstantValue();
      return constantValue != null && constantValue.equals(xpFlagName);
    }
    return false;
  }

  /**
   * Checks for {@link LiteralTree} and the flag name
   *
   * @return True if matches. Otherwise false
   */
  private boolean isLiteralTreeAndMatchesFlagName(ExpressionTree arg) {
    return arg instanceof LiteralTree
        && ((LiteralTree) arg).getValue() != null
        && ((LiteralTree) arg).getValue().equals(xpFlagName);
  }

  private String stripBraces(String s) {
    if (s.startsWith("{")) {
      s = s.substring(1);
      if (s.endsWith("}")) {
        s = s.substring(0, s.length() - 1);
      }
    }
    return s.trim();
  }

  /* this method checks for whether the enclosing source is already replaced.
   * e.g., when if(cond) { ... } is processed, it may be replaced with the then body.
   * but the checker subsequently processes cond, which can have its own matching.
   * The check for overLaps will ensure that this matching (and possible replacement) of
   * the internal expression does not happen.
   */
  private boolean overLaps(Tree t, VisitorState visitorState) {
    if (endPos != DONTCARE && visitorState.getEndPosition(t) <= endPos) {
      return true;
    } else {
      endPos = DONTCARE;
      return false;
    }
  }

  /* Evaluate the expression by handling various expression kinds.
   * Ensure that the appropriate XP API is also evaluated in the process.
   */
  private Value evalExpr(ExpressionTree tree, VisitorState state) {

    if (tree == null) {
      return Value.BOT;
    }

    Kind k = tree.getKind();

    if (k.equals(Kind.PARENTHESIZED)) {
      ParenthesizedTree pt = (ParenthesizedTree) tree;
      Value v = evalExpr(pt.getExpression(), state);
      return v;
    }

    if (k.equals(Kind.BOOLEAN_LITERAL)) {
      LiteralTree lt = (LiteralTree) tree;
      if (lt.getValue().equals(Boolean.TRUE)) {
        return Value.TRUE;
      }
      if (lt.getValue().equals(Boolean.FALSE)) {
        return Value.FALSE;
      }
    }

    if (k.equals(Kind.LOGICAL_COMPLEMENT)) {
      UnaryTree ut = (UnaryTree) tree;
      Value e = evalExpr(ut.getExpression(), state);
      if (e.equals(Value.FALSE)) {
        return Value.TRUE;
      }
      if (e.equals(Value.TRUE)) {
        return Value.FALSE;
      }
    }

    if (k.equals(Kind.METHOD_INVOCATION)) {
      API api = getXPAPI(tree, state);
      if (api.equals(API.IS_TREATED)) {
        return isTreated ? Value.TRUE : Value.FALSE;
      } else if (api.equals(API.IS_CONTROL)) {
        return isTreated ? Value.FALSE : Value.TRUE;
      } else if (api.equals(API.IS_TREATMENT_GROUP_CHECK)) {
        return evalTreatmentGroupCheck((MethodInvocationTree) tree) ? Value.TRUE : Value.FALSE;
      }
    }

    if (k.equals(Kind.CONDITIONAL_AND) || k.equals(Kind.CONDITIONAL_OR)) {
      BinaryTree bt = (BinaryTree) tree;

      Value l = evalExpr(bt.getLeftOperand(), state);
      Value r = evalExpr(bt.getRightOperand(), state);

      if (k.equals(Kind.CONDITIONAL_OR)) {
        if (l.equals(Value.TRUE) || r.equals(Value.TRUE)) {
          return Value.TRUE;
        }

        if (l.equals(Value.FALSE) && r.equals(Value.FALSE)) {
          return Value.FALSE;
        }

      } else if (k.equals(Kind.CONDITIONAL_AND)) {
        if (l.equals(Value.TRUE) && r.equals(Value.TRUE)) {
          return Value.TRUE;
        }

        if (l.equals(Value.FALSE) || r.equals(Value.FALSE)) {
          return Value.FALSE;
        }
      }
    }

    return Value.BOT;
  }

  /**
   * A utility method that simulated evaluation of isInTreatmentGroup and similar methods.
   *
   * @param methodInvocationTree The method call to isInTreatmentGroup
   * @return true if the second argument matches the treatmentGroup passed to this checker
   */
  private boolean evalTreatmentGroupCheck(MethodInvocationTree methodInvocationTree) {
    Preconditions.checkArgument(
        methodInvocationTree.getArguments().size() == 2,
        "Treatment group checks (e.g. isInTreatmentGroup) must take two arguments");
    ExpressionTree arg = methodInvocationTree.getArguments().get(1);
    Symbol argSym = ASTHelpers.getSymbol(arg);
    return (argSym != null && argSym.toString().toLowerCase().equals(treatmentGroup));
  }

  /* A utility method to update code corresponding to an expression
   *  used for various expression kinds
   */
  private Description updateCode(
      Value v, ExpressionTree tree, ExpressionTree expr, VisitorState state) {
    if (v.equals(Value.TRUE) || v.equals(Value.FALSE)) {
      SuggestedFix.Builder fixBuilder = handleSpecificAPIPatterns(state);
      if (fixBuilder.isEmpty()) {
        String replacementString = v.equals(Value.TRUE) ? TRUE : FALSE;
        fixBuilder = SuggestedFix.builder().replace(expr, replacementString);
        endPos = state.getEndPosition(expr);
      }
      decrementAllSymbolUsages(expr, state, fixBuilder);
      return buildDescription(tree).addFix(fixBuilder.build()).build();
    }
    return Description.NO_MATCH;
  }

  /**
   * This method picks up the unnecessary test method as configured in properties.json and converts
   * them into a Error-prone AST Matcher and then deletes the containing AST statement.
   *
   * @param state The visitor state of the statement to be handled
   * @return Suggestion Fix for deleting the statement containing a unnecessary test method
   *     invocation
   */
  private SuggestedFix.Builder handleSpecificAPIPatterns(VisitorState state) {
    MethodInvocationTree enclosingMit =
        ASTHelpers.findEnclosingNode(state.getPath(), MethodInvocationTree.class);

    ExpressionStatementTree enclosingEst =
        ASTHelpers.findEnclosingNode(state.getPath(), ExpressionStatementTree.class);

    if (enclosingMit != null
        && enclosingEst != null
        && config
            .getUnnecessaryTestMethodRecords()
            .stream()
            .map(
                method ->
                    method.isStatic()
                        ? method
                            .getReceiverType()
                            .map(r -> staticMethod().onClass(r))
                            .orElseGet(() -> staticMethod().anyClass())
                        : method
                            .getReceiverType()
                            .map(r -> instanceMethod().onExactClass(r))
                            .orElseGet(() -> instanceMethod().anyClass()))
            .anyMatch(matcher -> matcher.matches(enclosingMit, state))) {
      endPos = state.getEndPosition(enclosingMit);
      return SuggestedFix.builder().delete(enclosingEst);
    }
    return SuggestedFix.builder();
  }

  private boolean isTreatmentGroupEnum(Symbol.ClassSymbol enumSym) {
    // Filter out some generic names, like CONTROL to make sure we don't match the wrong
    // TreatmentGroup
    if (COMMON_GROUP_NAMES.contains(treatmentGroup)) {
      return false;
    }
    for (Symbol fsym : enumSym.getEnclosedElements()) {
      if (fsym.getKind().equals(ElementKind.ENUM_CONSTANT)
          && fsym.name.toString().toLowerCase().equals(treatmentGroup)) {
        return true;
      }
    }
    return false;
  }

  @Override
  public Description matchClass(ClassTree classTree, VisitorState visitorState) {
    if (shouldSkip(visitorState)) return Description.NO_MATCH;
    Symbol.ClassSymbol classSymbol = ASTHelpers.getSymbol(classTree);
    if (classSymbol.getKind().equals(ElementKind.ENUM) && isTreatmentGroupEnum(classSymbol)) {
      treatmentGroupsEnum = classSymbol.fullname.toString();
      if (classSymbol.getNestingKind().isNested()) {
        return buildDescription(classTree).addFix(SuggestedFix.delete(classTree)).build();
      } else {
        String emptyEnum =
            DELETE_REQUEST_COMMENT + "enum " + classSymbol.getSimpleName().toString() + " { }";
        return buildDescription(classTree)
            .addFix(SuggestedFix.replace(classTree, emptyEnum))
            .build();
      }
    }
    return Description.NO_MATCH;
  }

  // Likely worth fixing, not sure of a better way to match import FQNs:
  @SuppressWarnings("TreeToString")
  @Override
  public Description matchImport(ImportTree importTree, VisitorState visitorState) {
    if (shouldSkip(visitorState)) return Description.NO_MATCH;
    if (importTree.isStatic()) {
      Tree importIdentifier = importTree.getQualifiedIdentifier();
      if (importIdentifier.getKind().equals(Kind.MEMBER_SELECT)) {
        MemberSelectTree memberSelectTree = (MemberSelectTree) importIdentifier;
        String[] fullyQualifiedNameParts = memberSelectTree.getIdentifier().toString().split("\\.");
        Preconditions.checkArgument(
            fullyQualifiedNameParts.length > 0,
            "String.split should never produce a zero-length array. "
                + "The worst case should be the original string wrapped in a length 1 array.");
        String importSimpleName = fullyQualifiedNameParts[fullyQualifiedNameParts.length - 1];
        if (importSimpleName.equals(xpFlagName)
            || (treatmentGroupsEnum != null
                && memberSelectTree.getExpression().toString().startsWith(treatmentGroupsEnum))) {
          return buildDescription(importTree)
              .addFix(SuggestedFix.replace(importTree, "", 0, 1))
              .build();
        } else if (treatmentGroup.length() > 0 && treatmentGroupsEnum == null) {
          // Check if this import is for values in the same enum that includes the treatmentGroup
          Symbol importSymbol = ASTHelpers.getSymbol(memberSelectTree.getExpression());
          if (importSymbol.getKind().equals(ElementKind.ENUM)
              && isTreatmentGroupEnum((Symbol.ClassSymbol) importSymbol)) {
            treatmentGroupsEnum = ((Symbol.ClassSymbol) importSymbol).fullname.toString();
            return buildDescription(importTree)
                .addFix(SuggestedFix.replace(importTree, "", 0, 1))
                .build();
          }
        } else {
          // Check if this import matches an enum whose field value matches the flag name to remove
          String enumName = memberSelectTree.getIdentifier().toString();
          Symbol importSymbol = ASTHelpers.getSymbol(memberSelectTree.getExpression());

          if (importSymbol.getKind().equals(ElementKind.ENUM)
              && isEnumConstantMatchingFlagName(
                  enumName, (Symbol.ClassSymbol) importSymbol, visitorState)) {
            return buildDescription(importTree)
                .addFix(SuggestedFix.replace(importTree, "", 0, 1))
                .build();
          }
        }
      }
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchAssignment(AssignmentTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;
    if (tree.getExpression().getKind().equals(Kind.BOOLEAN_LITERAL) || overLaps(tree, state)) {
      return Description.NO_MATCH;
    }
    Value x = evalExpr(tree.getExpression(), state);
    return updateCode(x, tree, tree.getExpression(), state);
  }

  @Override
  public Description matchUnary(UnaryTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }
    Value x = evalExpr(tree, state);
    return updateCode(x, tree, tree, state);
  }

  @Override
  public Description matchMethodInvocation(MethodInvocationTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }

    Value x = evalExpr(tree, state);
    return updateCode(x, tree, tree, state);
  }

  // Pretty sure the Tree.toString() API is our best option here, but be aware of the issues listed
  // in: https://errorprone.info/bugpattern/TreeToString
  // e.g. comments and whitespace might be removed, which could be retrieved through
  // `VisitorState#getSourceForNode`
  // (but that makes the actual AST rewriting harder).
  @SuppressWarnings("TreeToString")
  @Override
  public Description matchBinary(BinaryTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }

    Value x = evalExpr(tree, state);
    Description d = updateCode(x, tree, tree, state);
    if (!d.equals(Description.NO_MATCH)) {
      return d;
    }

    ExpressionTree deletedSubTree = null;
    ExpressionTree remainingSubTree = null;
    Value l = evalExpr(tree.getLeftOperand(), state);
    Value r = evalExpr(tree.getRightOperand(), state);
    if (tree.getKind().equals(Kind.CONDITIONAL_AND)) {
      if (l.equals(Value.TRUE)) {
        deletedSubTree = tree.getLeftOperand();
        remainingSubTree = tree.getRightOperand();
      } else if (r.equals(Value.TRUE)) {
        deletedSubTree = tree.getRightOperand();
        remainingSubTree = tree.getLeftOperand();
      }
    } else if (tree.getKind().equals(Kind.CONDITIONAL_OR)) {
      if (l.equals(Value.FALSE)) {
        deletedSubTree = tree.getLeftOperand();
        remainingSubTree = tree.getRightOperand();
      } else if (r.equals(Value.FALSE)) {
        deletedSubTree = tree.getRightOperand();
        remainingSubTree = tree.getLeftOperand();
      }
    }

    if (deletedSubTree != null) {
      Preconditions.checkNotNull(
          remainingSubTree, "deletedSubTree != null => remainingSubTree !=null here.");
      Tree parent = state.getPath().getParentPath().getLeaf();
      Tree replacee = parent instanceof ParenthesizedTree ? parent : tree;
      Description.Builder builder = buildDescription(replacee);
      SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
      fixBuilder.replace(replacee, remainingSubTree.toString());
      decrementAllSymbolUsages(deletedSubTree, state, fixBuilder);
      builder.addFix(fixBuilder.build());

      endPos = state.getEndPosition(tree);
      return builder.build();
    }

    return Description.NO_MATCH;
  }

  @Override
  public Description matchExpressionStatement(ExpressionStatementTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }

    if (tree.getExpression().getKind().equals(Kind.METHOD_INVOCATION)) {
      MethodInvocationTree mit = (MethodInvocationTree) tree.getExpression();
      API api = getXPAPI(mit, state);
      if (api.equals(API.DELETE_METHOD)
          || api.equals(API.SET_TREATED)
          || api.equals(API.SET_CONTROL)) {
        Description.Builder builder = buildDescription(tree);
        SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
        fixBuilder.delete(tree);
        decrementAllSymbolUsages(tree, state, fixBuilder);
        builder.addFix(fixBuilder.build());
        endPos = state.getEndPosition(tree);
        return builder.build();
      }
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchReturn(ReturnTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }
    ExpressionTree et = tree.getExpression();
    if (et != null && et.getKind().equals(Kind.BOOLEAN_LITERAL)) {
      return Description.NO_MATCH;
    }

    Value x = evalExpr(et, state);
    boolean update = false;
    String replacementString = EMPTY;

    if (x.equals(Value.TRUE)) {
      update = true;
      replacementString = TRUE;
    } else if (x.equals(Value.FALSE)) {
      update = true;
      replacementString = FALSE;
    }

    if (update) {
      Description.Builder builder = buildDescription(tree);
      SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
      fixBuilder.replace(et, replacementString);
      decrementAllSymbolUsages(et, state, fixBuilder);
      builder.addFix(fixBuilder.build());
      endPos = state.getEndPosition(tree);
      return builder.build();
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchVariable(VariableTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;

    Symbol identifier = FindIdentifiers.findIdent(xpFlagName, state);

    // Check if this is the flag definition and remove it.
    if (identifier != null
        && identifier.isEnum()
        && identifier.equals(ASTHelpers.getSymbol(tree))) {
      xpSym = identifier;
      return removeEnumValue(
          tree, state, state.getSourceForNode(tree), isOnlyEnumConstant(xpSym, state));
    } else if (identifier == null
        && ASTHelpers.getSymbol(tree) != null
        && xpFlagName.equals(ASTHelpers.getSymbol(tree).getConstantValue())) {
      return buildDescription(tree).addFix(SuggestedFix.delete(tree)).build();
    }

    Symbol sym = ASTHelpers.getSymbol(tree);

    // Also checks if this is the flag definition. However, this is for the case where xpFlagName
    // does not match the enum constant itself, but instead, a String value in the enum constructor
    // (e.g. STALE_FLAG("stale.flag"), where xpFlagName is "stale.flag")
    if (sym != null && sym.isEnum() && tree.getInitializer().getKind() == Tree.Kind.NEW_CLASS) {
      // Enum constructor
      NewClassTree nct = (NewClassTree) tree.getInitializer();
      String enumClassName = sym.enclClass().getSimpleName().toString();
      ImmutableCollection<PiranhaEnumRecord> enumRecords =
          this.config.getEnumRecordsForName(enumClassName);

      boolean containsFlagName =
          enumConstructorArgsContainsFlagNameMatcher(enumRecords).matches(nct, state);
      if (containsFlagName) {
        return removeEnumValue(
            tree, state, state.getSourceForNode(tree), isOnlyEnumConstant(sym, state));
      }
    }

    // We also match the enum constant previous to the one being removed in some cases,
    // to fix delimiters.
    if (sym != null && sym.isEnum()) {
      EnumEnding enumEnding = getEndingOfLastEnumConstantIfRemoved(sym, state);

      if (enumEnding == EnumEnding.IGNORE) {
        return Description.NO_MATCH;
      }

      // The next enum constant in the list will be removed by Piranha
      // Let's replace the current enum constant's ending with the previous one
      return buildDescription(tree)
          .addFix(
              SuggestedFix.replace(tree, state.getSourceForNode(tree) + enumEnding.getChar(), 0, 1))
          .build();
    }

    return Description.NO_MATCH;
  }

  /** Represents the character at the end of an enum constant */
  enum EnumEnding {
    // Ignore the result of the trailing enum character, whether we know what it is or not
    IGNORE(null),
    COMMA(","),
    SEMICOLON(";"),
    NONE("");
    @Nullable final String endingChar;

    @Nullable
    String getChar() {
      return endingChar;
    }

    EnumEnding(@Nullable final String endingChar) {
      this.endingChar = endingChar;
    }
  }

  private Stream<VariableTree> getEnumConstants(ClassTree enumClassTree) {
    return enumClassTree
        .getMembers()
        .stream()
        .filter(member -> member.getKind() == Kind.VARIABLE)
        .map(member -> ((VariableTree) member))
        .filter(member -> ASTHelpers.getSymbol(member).getKind() == ElementKind.ENUM_CONSTANT);
  }

  private boolean isOnlyEnumConstant(Symbol enumSym, VisitorState state) {
    ClassTree enumClassTree = ASTHelpers.findClass(enumSym.enclClass(), state);

    if (enumClassTree == null) {
      return false;
    }

    // True if there is only a single enum constant for this enum
    return getEnumConstants(enumClassTree).count() == 1;
  }

  /**
   * This "looks ahead" at the next enum constant in an enum definition to see if it will be cleaned
   * up by Piranha. That way, we can know what kind of character to append to the end of the
   * previous enum constant after the last one is removed. This is important for enums that have non
   * enum constant members after their enum constants, such as constructors, methods, and fields.
   */
  private EnumEnding getEndingOfLastEnumConstantIfRemoved(Symbol enumSym, VisitorState state) {
    ClassTree enumClassTree = ASTHelpers.findClass(enumSym.enclClass(), state);

    if (enumClassTree == null) {
      return EnumEnding.IGNORE;
    }

    // Find enum initializer members only
    List<VariableTree> enumConstants = getEnumConstants(enumClassTree).collect(Collectors.toList());

    // Get second to last index
    //
    // The reason to get the second to last index, is that we need to alter the second to last enum
    // constant ending if the last constant is being removed, as the second to last enum constant
    // will now become the last constant.
    //
    // In Java, the last enum constant is be required to have a different ending than other enum
    // constants in the enum if the enum has non-enum constant members
    //
    // We do not need to alter any other enum constants because they will either never become the
    // last constant [0 ... size - 3] or will be removed [size - 1].
    int index = enumConstants.size() - 2;

    // Check if the second to last enum is the current enum being processed
    if (index < 0 || !enumConstants.get(index).getName().equals(enumSym.getSimpleName())) {
      return EnumEnding.IGNORE;
    }

    VariableTree nextConstant = enumConstants.get(index + 1);
    String nextEnumConstantSource = state.getSourceForNode(nextConstant);
    // The enclosingEnumSource is the source code of the enum that will contain both the current
    // enum constant and the next enum constant
    String enclosingEnumSource = state.getSourceForNode(state.getPath().getParentPath().getLeaf());

    // We need access to the source to remove the enum constant in the first place
    if (nextEnumConstantSource == null || enclosingEnumSource == null) {
      return EnumEnding.IGNORE;
    }

    // Make sure the last enum constant in the member list will actually be removed and cleaned up
    // If not, we don't want to alter the prior enum constant
    //
    // These are the same checks that would run from the point in the AST were we at the enum
    // constant that is being removed instead of the enum constant right before. We're skipping
    // around the tree a bit here.
    //
    // The reason for re-checking if we are going to clean up the last enum constant when we
    // encounter the second to last enum constant, rather than doing the check when we encounter the
    // last enum constant, is because we are altering the syntax of the second to last enum
    // constant. It is easier to add a SuggestedFix with the design of error-prone's BugChecker at
    // the time you encounter that piece of code you are altering, than to attempt to add a fix
    // outside of encountering that piece of code in the syntax tree traversal.
    if (!(xpFlagName.equals(nextConstant.getName().toString())
        || isEnumConstantMatchingFlagName(
            nextConstant.getName().toString(), enumSym.enclClass(), state))) {
      return EnumEnding.IGNORE;
    }

    String varAsStrWithComma = nextEnumConstantSource + ",";
    String varAsStrWithSemicolon = nextEnumConstantSource + ";";
    if (enclosingEnumSource.contains(varAsStrWithComma)) {
      return EnumEnding.COMMA;
    } else if (enclosingEnumSource.contains(varAsStrWithSemicolon)) {
      return EnumEnding.SEMICOLON;
    } else {
      return EnumEnding.NONE;
    }
  }

  /**
   * Remove the flag symbol. This only works because the error prone patch is applied once after all
   * files have been analyzed, otherwise targets that use the flag but haven't been cleaned up would
   * be broken. We use replace with a position adjustment, to get rid of the trailing "," or ";" if
   * present on the parent.
   */
  private Description removeEnumValue(
      VariableTree tree, VisitorState state, String varName, boolean isSingleEnumConstant) {
    String enumAsStr = state.getSourceForNode(state.getPath().getParentPath().getLeaf());
    String varAsStrWithComma = varName + ",";
    String varAsStrWithSemicolon = varName + ";";
    if (enumAsStr != null
        && (enumAsStr.contains(varAsStrWithComma)
            || (enumAsStr.contains(varAsStrWithSemicolon) && !isSingleEnumConstant))) {
      return buildDescription(tree).addFix(SuggestedFix.replace(tree, "", 0, 1)).build();
    } else {
      // Fallback for single/last enum variable detection
      //
      // Also if we remove the only enum constant left, and the last enum constant ends in a
      // semi-colon, we will leave that semi-colon to make sure we still have compilable Java if
      // there are non-enum-constant fields or methods in the enum class
      return buildDescription(tree).addFix(SuggestedFix.delete(tree)).build();
    }
  }

  private void recursiveScanTestMethodStats(
      BlockTree blockTree, VisitorState state, TestMethodCounters counters, int depth) {
    for (StatementTree statement : blockTree.getStatements()) {
      if (statement.getKind().equals(Tree.Kind.BLOCK)) {
        recursiveScanTestMethodStats(blockTree, state, counters, depth + 1);
      } else if (statement.getKind().equals(Kind.EXPRESSION_STATEMENT)) {
        counters.statements += 1;
        ExpressionTree expr = ((ExpressionStatementTree) statement).getExpression();
        if (!expr.getKind().equals(Kind.METHOD_INVOCATION)) {
          continue;
        }
        MethodInvocationTree mit = (MethodInvocationTree) expr;
        if (!mit.getMethodSelect().getKind().equals(Tree.Kind.MEMBER_SELECT)) {
          continue; // Can't resolve to API call
        }
        // We scan for config method records of type SET_* here directly, since getXPAPI(...) will
        // resolve
        // only when the flag name matches, and we want to verify that no calls are being made to
        // set
        // unrelated flags (i.e. count them in counters.allSetters).
        for (PiranhaMethodRecord methodRecord : config.getMethodRecordsForName(mit, state)) {
          if (methodRecord.getApiType().equals(XPFlagCleaner.API.SET_TREATED)) {
            counters.allSetters += 1;
            // If the test is asking for the flag in treated condition, but we are setting it to
            // control (in a top level statement), then this test is obsolete.
            // Remember that we are scanning for all setters, however, so now we must check that
            // call is for the
            // flag being passed to Piranha.
            if (!isTreated && depth == 0) {
              if (getXPAPI(mit, state).equals(API.SET_TREATED)) {
                counters.topLevelObsoleteSetters += 1;
              }
            }
          } else if (methodRecord.getApiType().equals(XPFlagCleaner.API.SET_CONTROL)) {
            counters.allSetters += 1;
            // Analogous to the case above, but now we are checking if the test is asking for the
            // flag in the control
            // condition, while we are setting it to treated everywhere
            if (isTreated && depth == 0) {
              if (getXPAPI(mit, state).equals(API.SET_CONTROL)) {
                counters.topLevelObsoleteSetters += 1;
              }
            }
          }
        }
      } else {
        counters.statements += 1;
      }
    }
  }

  /**
   * This method implements the clean up heuristic requested by the
   * tests.clean_by_setters_heuristic.enabled option.
   *
   * <p>The heuristic uses calls to set_treated/set_control API methods.
   *
   * <p>Under this heuristic, a test method is cleaned up iff:
   *
   * <ol>
   *   <li>The method has a call to a set_treated/set_control API method as a top-level (unnested)
   *       statement, where:
   *       <ol>
   *         <li>The flag being set matches the flag being cleaned up by Piranha
   *         <li>The test is checking for the opposite of the treatment condition being set (i.e.
   *             set_treated for a flag being refactored globally as control, or set_control for a
   *             flag being globally refactored as treated)
   *         <li>This call can be repeated
   *       </ol>
   *   <li>The method has no other calls for set_treated/set_control API methods for other flags or
   *       conditions
   *   <li>The method length does not exceed tests.clean_by_setters_heuristic.lines_limit (long unit
   *       tests are more likely to be testing multiple things)
   *       <ol>
   *
   * @param tree A method tree that has already been detected as being an unit test method.
   * @return whether the method should be deleted based on the heuristic above.
   */
  private boolean shouldCleanBySetters(MethodTree tree, VisitorState state) {
    // This assumes that the method was already detected as a unit test and the corresponding
    // heuristic is enabled
    TestMethodCounters counters = new TestMethodCounters();
    // Count statements, all setter calls, and relevant top-level obsolete setters for the current
    // flag
    recursiveScanTestMethodStats(tree.getBody(), state, counters, 0);
    if (config.testMethodCleanupSizeLimit() < counters.statements) {
      // Skip, test method too large.
      return false;
    }
    if (counters.topLevelObsoleteSetters > 0) {
      if (config.shouldIgnoreOtherSettersWhenCleaningTests()) {
        // Ignore other setter calls
        return true;
      } else if (counters.topLevelObsoleteSetters == counters.allSetters) {
        // All calls to flag setting methods in this test are in the top level scope and setting the
        // flag to a now impossible value. The whole test should be deleted as per this heuristic.
        return true;
      }
    }
    // Heuristic doesn't match, unsafe to clean
    return false;
  }

  @Override
  public Description matchMethod(MethodTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;

    boolean deleteMethod = false;
    List<AnnotationTree> deletableAnnotations = new ArrayList<>();
    // Deletable identifiers used as part of the argument to an annotation, for when the entire
    // annotation doesn't need to be deleted
    List<ExpressionTree> deletableIdentifiers = new ArrayList<>();
    final ImmutableSet<ResolvedTestAnnotation> resolvedTestAnnotations =
        config.resolveTestAnnotations(tree, state);
    if (resolvedTestAnnotations.size() != 0) {
      // If there are any test annotations for this method, then we clean up based on those. We do
      // this even if
      // the method is not otherwise a recognized test method (e.g. marked @Test) and we ignore all
      // other
      // applicable clean up heuristics
      for (ResolvedTestAnnotation resolved : resolvedTestAnnotations) {
        Set<AnnotationArgument> matchedFlagsWorkingSet = new HashSet<>();
        for (AnnotationArgument testedFlag : resolved.getFlags()) {
          if (testedFlag.getValue().equals(xpFlagName)) {
            if (isTreated == resolved.isTreated()) {
              // Annotation requests the same treatment state as what Piranha is setting the flag to
              matchedFlagsWorkingSet.add(testedFlag);
            } else {
              // Annotation (and therefore test method) requests a different (now impossible)
              // treatment, compared to what Piranha is setting the flag to.
              deleteMethod = true;
            }
          }
        }
        // Should we delete the full annotation, or specific flags within it? Depends if all
        // flags mentioned within the annotation have been matched or not:
        if (matchedFlagsWorkingSet.size() == resolved.getFlags().size()) {
          deletableAnnotations.add(resolved.getSourceTree());
        } else {
          // Only remove the matched flag references, but preserve the annotation and its
          // references to remaining flags
          matchedFlagsWorkingSet.forEach(arg -> deletableIdentifiers.add(arg.getSourceTree()));
        }
        matchedFlagsWorkingSet.clear();
      }
    } else if (config.shouldCleanTestMethodsByContent()
        && PiranhaUtils.isUnitTestMethod(tree, state)) {
      deleteMethod = shouldCleanBySetters(tree, state);
    }
    // Early exit for no changes required:
    if (!deleteMethod && deletableAnnotations.size() == 0 && deletableIdentifiers.size() == 0) {
      return Description.NO_MATCH;
    }
    // Process refactoring
    Description.Builder builder = buildDescription(tree);
    SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
    // Check first if the whole method must go away
    if (deleteMethod) {
      fixBuilder.delete(tree);
      decrementAllSymbolUsages(tree, state, fixBuilder);
      skipWithin(state.getPath()); // Deleting full method, skip refactorings within it
    } else {
      // Otherwise, we might still need to clean up individual obsolete test annotations and flag
      // references within multi-flag annotations.
      // Note that the AST elements referenced by deletableAnnotations and deletableIdentifiers
      // should not overlap, given the logic above, so deleting each independently is safe.
      for (ExpressionTree expr : deletableIdentifiers) {
        fixBuilder = deleteExprWithComma(state, expr, resolvedTestAnnotations, fixBuilder);
        decrementAllSymbolUsages(expr, state, fixBuilder);
      }
      for (AnnotationTree at : deletableAnnotations) {
        fixBuilder.delete(at);
        decrementAllSymbolUsages(at, state, fixBuilder);
      }
    }
    builder.addFix(fixBuilder.build());
    return builder.build();
  }

  @Override
  public Description matchConditionalExpression(
      ConditionalExpressionTree tree, VisitorState state) {
    if (shouldSkip(state)) return Description.NO_MATCH;
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }
    ExpressionTree et = tree.getCondition();
    Value x = evalExpr(et, state);
    String replacementString = EMPTY;

    boolean update = false;
    ExpressionTree removedBranch = null;
    if (x.equals(Value.TRUE)) {
      update = true;
      replacementString = state.getSourceForNode(tree.getTrueExpression());
      removedBranch = tree.getFalseExpression();
    } else if (x.equals(Value.FALSE)) {
      update = true;
      replacementString = state.getSourceForNode(tree.getFalseExpression());
      removedBranch = tree.getTrueExpression();
    }

    if (update) {
      Preconditions.checkNotNull(removedBranch, "update => removedBranch != null here.");
      Description.Builder builder = buildDescription(tree);
      SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
      fixBuilder.replace(tree, stripBraces(replacementString));
      decrementAllSymbolUsages(et, state, fixBuilder);
      decrementAllSymbolUsages(removedBranch, state, fixBuilder);
      builder.addFix(fixBuilder.build());
      endPos = state.getEndPosition(tree);
      return builder.build();
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchIf(IfTree ifTree, VisitorState visitorState) {
    if (shouldSkip(visitorState)) return Description.NO_MATCH;
    if (overLaps(ifTree, visitorState)) {
      return Description.NO_MATCH;
    }

    // Ignore if this is part of an else-if clause, we will process it all at once
    // when we get to the topmost if.
    Tree parentTree = visitorState.getPath().getParentPath().getLeaf();
    if (parentTree.getKind().equals(Kind.IF)) {
      Tree parentElseChild = ((IfTree) parentTree).getElseStatement();
      if (parentElseChild != null && parentElseChild.equals(ifTree)) {
        return Description.NO_MATCH;
      }
    }

    ParenthesizedTree parenTree = (ParenthesizedTree) ifTree.getCondition();
    Value x = evalExpr(parenTree.getExpression(), visitorState);
    boolean update = false;
    String replacementString = EMPTY;
    String replacementPrefix = EMPTY;
    boolean lastStmtIsReturn = false;
    Set<StatementTree> removedBranches = new LinkedHashSet<StatementTree>();
    // This code simplifies a nested if {...} (else if {...})* (else {...})? three all at once
    IfTree subIfTree = ifTree;
    boolean recurse;
    do {
      recurse = false;
      StatementTree elseStatement = subIfTree.getElseStatement();
      if (x.equals(Value.TRUE)) {
        update = true;
        if (elseStatement != null) {
          removedBranches.add(elseStatement);
        }
        replacementString = visitorState.getSourceForNode(subIfTree.getThenStatement());
        lastStmtIsReturn = endsWithReturn(ifTree.getThenStatement());
      } else if (x.equals(Value.FALSE)) {
        update = true;
        if (elseStatement != null) {
          removedBranches.add(subIfTree.getThenStatement());
          replacementString = visitorState.getSourceForNode(elseStatement);
          if (elseStatement.getKind().equals(Kind.IF)) {
            // Keep going, in case we can eliminate more of the branches of the
            // nested if.
            recurse = true;
            subIfTree = (IfTree) elseStatement;
            ParenthesizedTree pT = (ParenthesizedTree) subIfTree.getCondition();
            x = evalExpr(pT, visitorState);
          } else {
            lastStmtIsReturn = endsWithReturn(subIfTree.getElseStatement());
          }
        }
      } else {
        // The condition doesn't simplify to a constant, but the condition to some nested "else if"
        // might.
        if (elseStatement != null && elseStatement.getKind().equals(Kind.IF)) {
          // Copy the initial if condition (don't mark as needing update yet)
          replacementPrefix +=
              "if " + visitorState.getSourceForNode(subIfTree.getCondition()) + " ";
          replacementPrefix +=
              visitorState.getSourceForNode(subIfTree.getThenStatement()) + " else ";
          // Then recurse on the else case
          recurse = true;
          subIfTree = (IfTree) elseStatement;
          ParenthesizedTree pT = (ParenthesizedTree) subIfTree.getCondition();
          x = evalExpr(pT, visitorState);
        }
      }
    } while (recurse);

    if (update) {
      if (!replacementPrefix.equals(EMPTY)) {
        replacementString = replacementPrefix + replacementString;
      } else {
        replacementString = stripBraces(replacementString);
      }
      Description.Builder builder = buildDescription(ifTree);
      // We use SuggestedFix.Builder to AND-compose fixes. Note that calling
      // Description.Builder.addFix(...) multiple times is interpreted as OR-composing
      // multiple candidate fixes (i.e. "Fix by doing A or B or C" where we want
      // "Fix by doing A and B and C")
      SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
      if (lastStmtIsReturn) {
        // find the parent, and if it's a BlockTree, replace the if statement and delete any
        // subsequent statements
        Tree parent = visitorState.getPath().getParentPath().getLeaf();
        // note that parent may not be a block, e.g., if we have a parent if statement with no
        // braces for the body
        if (parent instanceof BlockTree) {
          BlockTree block = (BlockTree) parent;
          boolean foundIf = false;
          for (StatementTree stmt : block.getStatements()) {
            if (foundIf) {
              // We are past the if statement, so everything after this will be deleted,
              // decrement all usage counts accordingly
              decrementAllSymbolUsages(stmt, visitorState, fixBuilder);
              fixBuilder.delete(stmt);
            } else if (!stmt.equals(ifTree)) {
              // preceding statement, keep it
              continue;
            } else {
              // we reached the if
              for (StatementTree removedBranch : removedBranches) {
                decrementAllSymbolUsages(removedBranch, visitorState, fixBuilder);
              }
              decrementAllSymbolUsages(ifTree.getCondition(), visitorState, fixBuilder);
              fixBuilder.replace(ifTree, replacementString);
              // elide the remaining statements
              foundIf = true;
            }
          }
          // Usage counts already decremented above.
          endPos = visitorState.getEndPosition(block);
          return builder.addFix(fixBuilder.build()).build();
        }
      }
      fixBuilder.replace(ifTree, replacementString);
      for (StatementTree removedBranch : removedBranches) {
        decrementAllSymbolUsages(removedBranch, visitorState, fixBuilder);
      }
      decrementAllSymbolUsages(ifTree.getCondition(), visitorState, fixBuilder);
      endPos = visitorState.getEndPosition(ifTree);
      return builder.addFix(fixBuilder.build()).build();
    }

    return Description.NO_MATCH;
  }

  /** Is the statement a return statement, or is it a block that ends in a return statement? */
  private boolean endsWithReturn(StatementTree stmt) {
    if (stmt instanceof ReturnTree) {
      return true;
    }
    if (stmt instanceof BlockTree) {
      List<? extends StatementTree> statements = ((BlockTree) stmt).getStatements();
      return statements.size() > 0 && statements.get(statements.size() - 1) instanceof ReturnTree;
    }
    return false;
  }

  /** removes enum in annotation and the comma if there is more than one enum in it */
  private SuggestedFix.Builder deleteExprWithComma(
      VisitorState state,
      ExpressionTree expressionTree,
      ImmutableSet<ResolvedTestAnnotation> resolvedTestAnnotations,
      SuggestedFix.Builder fixBuilder) {
    for (ResolvedTestAnnotation resolvedTestAnnotation : resolvedTestAnnotations) {

      // remove the comma before or after the enum depending on the index
      int index =
          IntStream.range(0, resolvedTestAnnotation.getFlags().size())
              .filter(i -> resolvedTestAnnotation.getFlags().get(i).getValue().equals(xpFlagName))
              .findFirst()
              .orElse(-1);
      if (index != -1) {
        JCTree node = (JCTree) resolvedTestAnnotation.getSourceTree();
        int startAbsolute = node.getStartPosition();
        int lower = ((JCTree) expressionTree).getStartPosition() - startAbsolute;
        int upper = state.getEndPosition(expressionTree) - startAbsolute;
        CharSequence source = state.getSourceForNode(node);

        lower = getLower(source, lower, index, resolvedTestAnnotation.getFlags().size());

        upper = getUpper(source, upper);

        fixBuilder.replace(startAbsolute + lower, startAbsolute + upper, "");
      }
    }
    return fixBuilder;
  }

  private int getLower(CharSequence source, int lower, int index, int flagSize) {
    while (lower >= 0 && source.charAt(lower) != '{' && source.charAt(lower) != ',') {
      lower--;
    }
    // do not remove { or = neither the comma when it is not the last enum
    if (source.charAt(lower) == '{' || (index != flagSize - 1 && source.charAt(lower) == ',')) {
      lower++;
    }
    return lower;
  }

  private int getUpper(CharSequence source, int upper) {
    while (upper < source.length()
        && source.charAt(upper) != ','
        && source.charAt(upper) != '}'
        && source.charAt(upper) != ')') {
      upper++;
    }
    if (source.charAt(upper) == ',') {
      upper++;
    }
    return upper;
  }

  /**
   * A small tuple of counters for scanning unit tests to be deleted by the setters heuristic. See
   * {@link #shouldCleanBySetters(MethodTree, VisitorState)}
   */
  private static final class TestMethodCounters {
    public long statements;
    public int topLevelObsoleteSetters;
    public int allSetters;

    public TestMethodCounters() {
      this.statements = 0;
      this.topLevelObsoleteSetters = 0;
      this.allSetters = 0;
    }
  }
}
