package com.uber.piranha;

import static com.google.errorprone.BugPattern.Category.JDK;
import static com.google.errorprone.BugPattern.SeverityLevel.SUGGESTION;

import java.text.ParseException;
import java.util.Optional;

import com.google.auto.service.AutoService;
import com.google.errorprone.BugPattern;
import com.google.errorprone.ErrorProneFlags;
import com.google.errorprone.VisitorState;
import com.google.errorprone.bugpatterns.BugChecker;
import com.google.errorprone.fixes.SuggestedFix;
import com.google.errorprone.matchers.Description;
import com.google.errorprone.util.ASTHelpers;
import com.google.errorprone.util.FindIdentifiers;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.ReturnTree;
import com.sun.source.tree.LiteralTree;
import com.sun.source.tree.AssignmentTree;
import com.sun.source.tree.BinaryTree;
import com.sun.source.tree.ConditionalExpressionTree;
import com.sun.source.tree.ExpressionStatementTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.IfTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodInvocationTree;
import com.sun.source.tree.ParenthesizedTree;
import com.sun.source.tree.Tree;
import com.sun.source.tree.UnaryTree;
import com.sun.source.tree.Tree.Kind;
import com.sun.source.tree.VariableTree;
import com.sun.source.tree.AnnotationTree;
import com.sun.tools.javac.code.Symbol;
import java.io.IOException;

import java.nio.file.Files;
import java.nio.charset.Charset;
import java.nio.file.Paths;

import java.util.HashSet;
import java.util.Properties;

/** @author murali@uber.com (Murali Krishna Ramanathan) */
@AutoService(BugChecker.class)
@BugPattern(
  name = "Piranha",
  category = JDK,
  summary = "Cleans stale XP flags.",
  severity = SUGGESTION
)
public class XPFlagCleaner extends BugChecker
    implements BugChecker.AssignmentTreeMatcher,
        BugChecker.BinaryTreeMatcher,
        BugChecker.ConditionalExpressionTreeMatcher,
        BugChecker.ExpressionStatementTreeMatcher,
        BugChecker.IfTreeMatcher,
        BugChecker.MethodInvocationTreeMatcher,
        BugChecker.ReturnTreeMatcher,
        BugChecker.UnaryTreeMatcher,
        BugChecker.VariableTreeMatcher,
        BugChecker.MethodTreeMatcher {

  private static final int DONTCARE = -1;

  private String xpFlagName = "_xpflag_dummy";
  private final String TRUE = "true";
  private final String FALSE = "false";
  private final String EMPTY = "";

  /** Enumerator for different values returned by expression evaluation */
  private enum Value {
    TRUE,
    FALSE,
    BOT
  };

  /** Identify the appropriate API for experiments */
  private enum API {
    IS_TREATED,
    IS_CONTROL,
    DELETE_METHOD,
    UNKNOWN
  }

  private Symbol xpSym = null;
  private boolean isTreated = true;

  /**
   * when source is refactored, this specifies the end position for the refactoring until the
   * processing of the source reaches this position. otherwise, the value is -1.
   */
  private int endPos = DONTCARE;

  /* information provided in the config file */
  private final HashSet<String> treatedMethods = new HashSet<String>();
  private final HashSet<String> controlMethods = new HashSet<String>();
  private final HashSet<String> deleteMethods = new HashSet<String>();
  private final HashSet<String> handledAnnotations = new HashSet<String>();
  private String linkURL;

  /**
   * Copied from NullAway comment. Error Prone requires us to have an empty constructor for each
   * Plugin, in addition to the constructor taking an ErrorProneFlags object. This constructor
   * should not be used anywhere else.
   */
  public XPFlagCleaner() {}

  public XPFlagCleaner(ErrorProneFlags flags) throws ParseException {
    Optional<String> s = flags.get("Piranha:FlagName");
    if (s.isPresent()) {
      xpFlagName = s.get();
      isTreated = flags.getBoolean("Piranha:IsTreated").orElse(true);
    } else {
      throw new ParseException("Piranha:FlagName is missing", 0);
    }

    Optional<String> f = flags.get("Piranha:Config");
    if (f.isPresent()) {
      String configFile = f.get();

      try {
        Properties prop = new Properties();
        prop.load(Files.newBufferedReader(Paths.get(configFile), Charset.defaultCharset()));
        updateConfig(prop, "treatedMethods", treatedMethods);
        updateConfig(prop, "controlMethods", controlMethods);
        updateConfig(prop, "emptyMethods", deleteMethods);
        updateConfig(prop, "annotations", handledAnnotations);
        linkURL = prop.getProperty("linkURL");
      } catch (IOException fnfe) {
        throw new ParseException("Provided config file is not found", 0);
      } catch (Exception e) {
        throw new ParseException("Some other exception thrown while parsing config", 0);
      }
    } else {
      throw new ParseException("Piranha:Config is missing", 0);
    }
  }

  @Override
  public String linkUrl() {
    return linkURL;
  }

  private void updateConfig(Properties prop, String key, HashSet<String> hs) {
    String str = prop.getProperty(key);
    for (String s : str.split(",")) {
      hs.add(s);
    }
  }

  /* Returns the appropriate XP API, if any, as given by the expression */
  private API getXPAPI(ExpressionTree et) {
    et = ASTHelpers.stripParentheses(et);
    Kind k = et.getKind();
    if (k.equals(Tree.Kind.METHOD_INVOCATION)) {
      MethodInvocationTree mit = (MethodInvocationTree) et;
      if (!mit.getMethodSelect().getKind().equals(Kind.MEMBER_SELECT)) {
        return API.UNKNOWN;
      }

      if (mit.getArguments().size() == 1) {
        ExpressionTree arg = mit.getArguments().get(0);
        Symbol argSym = ASTHelpers.getSymbol(arg);
        if (argSym != null && (argSym.equals(xpSym) || argSym.toString().equals(xpFlagName))) {
          MemberSelectTree mst = (MemberSelectTree) mit.getMethodSelect();
          String methodName = mst.getIdentifier().toString();
          if (controlMethods.contains(methodName)) {
            return API.IS_CONTROL;
          } else if (treatedMethods.contains(methodName)) {
            return API.IS_TREATED;
          } else if (deleteMethods.contains(methodName)) {
            return API.DELETE_METHOD;
          }
        }
      }
    }
    return API.UNKNOWN;
  }

  private String stripBraces(String s) {
    if (s.startsWith("{")) {
      s = s.substring(1);
      if (s.endsWith("}")) {
        s = s.substring(0, s.length() - 1);
      }
    }
    return s;
  }

  /* this method checks for whether the enclosing source is already replaced.
   * e.g., when if(cond) { ... } is processed, it may be replaced with the then body.
   * but the checker subsequently processes cond, which can have its own matching.
   * The check for overLaps will ensure that this matching (and possible replacement) of
   * the internal expression does not happen.
   */
  private boolean overLaps(Tree t, VisitorState visitorState) {
    if (endPos != DONTCARE && visitorState.getEndPosition(t) < endPos) {
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
      API api = getXPAPI(tree);
      if (api.equals(API.IS_TREATED)) {
        return isTreated ? Value.TRUE : Value.FALSE;
      }
      if (api.equals(API.IS_CONTROL)) {
        return isTreated ? Value.FALSE : Value.TRUE;
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

  /* A utility method to update code corresponding to an expression
   *  used for various expression kinds
   */
  private Description updateCode(
      Value v, ExpressionTree tree, ExpressionTree expr, VisitorState state) {
    boolean update = false;
    String replacementString = "";

    if (v.equals(Value.TRUE)) {
      update = true;
      replacementString = TRUE;
    } else if (v.equals(Value.FALSE)) {
      update = true;
      replacementString = FALSE;
    }

    if (update) {
      Description.Builder builder = buildDescription(tree);
      builder.addFix(SuggestedFix.replace(expr, replacementString));
      endPos = state.getEndPosition(expr);
      return builder.build();
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchAssignment(AssignmentTree tree, VisitorState state) {
    if (tree.getExpression().getKind().equals(Kind.BOOLEAN_LITERAL) || overLaps(tree, state)) {
      return Description.NO_MATCH;
    }
    Value x = evalExpr(tree.getExpression(), state);
    return updateCode(x, tree, tree.getExpression(), state);
  }

  @Override
  public Description matchUnary(UnaryTree tree, VisitorState state) {
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }
    Value x = evalExpr(tree, state);
    return updateCode(x, tree, tree, state);
  }

  @Override
  public Description matchMethodInvocation(MethodInvocationTree tree, VisitorState state) {
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }

    Value x = evalExpr(tree, state);
    return updateCode(x, tree, tree, state);
  }

  @Override
  public Description matchBinary(BinaryTree tree, VisitorState state) {
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }

    Value x = evalExpr(tree, state);
    Description d = updateCode(x, tree, tree, state);
    if (!d.equals(Description.NO_MATCH)) {
      return d;
    }

    String replacementString = null;
    Value l = evalExpr(tree.getLeftOperand(), state);
    Value r = evalExpr(tree.getRightOperand(), state);
    if (tree.getKind().equals(Kind.CONDITIONAL_AND)) {
      if (l.equals(Value.TRUE)) {
        replacementString = tree.getRightOperand().toString();
      } else if (r.equals(Value.TRUE)) {
        replacementString = tree.getLeftOperand().toString();
      }
    } else if (tree.getKind().equals(Kind.CONDITIONAL_OR)) {
      if (l.equals(Value.FALSE)) {
        replacementString = tree.getRightOperand().toString();
      } else if (r.equals(Value.FALSE)) {
        replacementString = tree.getLeftOperand().toString();
      }
    }

    if (replacementString != null) {
      Description.Builder builder = buildDescription(tree);
      builder.addFix(SuggestedFix.replace(tree, replacementString));
      endPos = state.getEndPosition(tree);
      return builder.build();
    }

    return Description.NO_MATCH;
  }

  @Override
  public Description matchExpressionStatement(ExpressionStatementTree tree, VisitorState state) {
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }

    if (tree.getExpression().getKind().equals(Kind.METHOD_INVOCATION)) {
      MethodInvocationTree mit = (MethodInvocationTree) tree.getExpression();
      API api = getXPAPI(mit);
      if (api.equals(API.DELETE_METHOD)) {
        Description.Builder builder = buildDescription(tree);
        builder.addFix(SuggestedFix.delete(tree));
        endPos = state.getEndPosition(tree);
        return builder.build();
      }
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchReturn(ReturnTree tree, VisitorState state) {
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
      builder.addFix(SuggestedFix.replace(et, replacementString));
      endPos = state.getEndPosition(tree);
      return builder.build();
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchVariable(VariableTree tree, VisitorState state) {
    Symbol sym = FindIdentifiers.findIdent(xpFlagName, state);
    if (sym != null && sym.isEnum() && sym.equals(ASTHelpers.getSymbol(tree))) {
      xpSym = sym;
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchMethod(MethodTree tree, VisitorState state) {

    for (String name : handledAnnotations) {
      AnnotationTree at =
          ASTHelpers.getAnnotationWithSimpleName(tree.getModifiers().getAnnotations(), name);

      if (at != null) {
        for (ExpressionTree et : at.getArguments()) {
          if (et.getKind() == Kind.ASSIGNMENT) {
            AssignmentTree assn = (AssignmentTree) et;
            if (assn.getExpression().toString().endsWith(xpFlagName)) {
              Description.Builder builder = buildDescription(tree);
              if (isTreated) {
                builder.addFix(SuggestedFix.delete(at));
              } else {
                builder.addFix(SuggestedFix.delete(tree));
              }
              return builder.build();
            }
          }
        }
      }
    }

    return Description.NO_MATCH;
  }

  @Override
  public Description matchConditionalExpression(
      ConditionalExpressionTree tree, VisitorState state) {
    if (overLaps(tree, state)) {
      return Description.NO_MATCH;
    }
    ExpressionTree et = tree.getCondition();
    Value x = evalExpr(et, state);
    boolean update = false;
    String replacementString = EMPTY;

    if (x.equals(Value.TRUE)) {
      update = true;
      replacementString = state.getSourceForNode(tree.getTrueExpression());
    } else if (x.equals(Value.FALSE)) {
      update = true;
      replacementString = state.getSourceForNode(tree.getFalseExpression());
    }

    if (update) {
      Description.Builder builder = buildDescription(tree);
      builder.addFix(SuggestedFix.replace(tree, stripBraces(replacementString)));
      endPos = state.getEndPosition(tree);
      return builder.build();
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchIf(IfTree ifTree, VisitorState visitorState) {

    if (overLaps(ifTree, visitorState)) {
      return Description.NO_MATCH;
    }

    ParenthesizedTree parenTree = (ParenthesizedTree) ifTree.getCondition();
    Value x = evalExpr(parenTree.getExpression(), visitorState);
    boolean update = false;
    String replacementString = EMPTY;

    if (x.equals(Value.TRUE)) {
      update = true;
      replacementString = visitorState.getSourceForNode(ifTree.getThenStatement());
    } else if (x.equals(Value.FALSE)) {
      update = true;
      if (ifTree.getElseStatement() != null) {
        replacementString = visitorState.getSourceForNode(ifTree.getElseStatement());
      }
    }

    if (update) {
      Description.Builder builder = buildDescription(ifTree);
      builder.addFix(SuggestedFix.replace(ifTree, stripBraces(replacementString)));
      endPos = visitorState.getEndPosition(ifTree);
      return builder.build();
    }

    return Description.NO_MATCH;
  }
}
