/**
 *    Copyright (c) 2019 Uber Technologies, Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */
package com.uber.piranha;

import static com.google.errorprone.BugPattern.SeverityLevel.SUGGESTION;

import com.google.auto.service.AutoService;
import com.google.errorprone.BugPattern;
import com.google.errorprone.ErrorProneFlags;
import com.google.errorprone.VisitorState;
import com.google.errorprone.bugpatterns.BugChecker;
import com.google.errorprone.fixes.SuggestedFix;
import com.google.errorprone.matchers.Description;
import com.google.errorprone.util.ASTHelpers;
import com.google.errorprone.util.FindIdentifiers;
import com.sun.source.tree.ClassTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.ImportTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodInvocationTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.NewClassTree;
import com.sun.source.tree.Tree;
import com.sun.source.tree.Tree.Kind;
import com.sun.source.tree.VariableTree;
import com.sun.tools.javac.code.Symbol;
import java.util.List;
import java.util.Optional;

/** @author murali@uber.com (Murali Krishna Ramanathan) */
@AutoService(BugChecker.class)
@BugPattern(name = "PluginCleaner", summary = "Cleans stale plugins", severity = SUGGESTION)
public class PluginCleaner extends BugChecker
    implements BugChecker.ClassTreeMatcher,
        BugChecker.VariableTreeMatcher,
        BugChecker.NewClassTreeMatcher,
        BugChecker.MemberSelectTreeMatcher,
        BugChecker.MethodInvocationTreeMatcher,
        BugChecker.ImportTreeMatcher {

  /**
   * Used to defer initialization until after traversal has begun, since Error Prone eats all error
   * messages thrown inside the constructor.
   */
  private ErrorProneFlags flags = null;

  private String pluginFactory = "_plugin_dummy";
  private String pluginName = "_pluginName_dummy";
  private String helperClassName = "_helper_dummy";

  private String linkURL;

  public PluginCleaner() {}

  public PluginCleaner(ErrorProneFlags flags) {
    Optional<String> s = flags.get("PluginCleaner:PluginFactory");
    if (s.isPresent()) {
      pluginFactory = s.get();
      pluginName = flags.get("PluginCleaner:PluginName").get();
    }
  }

  @Override
  public String linkUrl() {
    return linkURL;
  }

  private Description processArguments(
      MethodInvocationTree tree, VisitorState state, MemberSelectTree mst) {

    List<? extends ExpressionTree> args = tree.getArguments();
    for (ExpressionTree e : args) {
      if (e.getKind().equals(Kind.NEW_CLASS)) {
        NewClassTree nct = (NewClassTree) e;
        if (nct.getIdentifier().toString().equals(pluginFactory)) {
          SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
          if (args.size() == 1) {
            if (mst != null) {
              /* for the example discussed in matchMemberSelect,
              perform the replace of a.b(..).c(..) to a.c(..) */
              String methodStr = tree.getMethodSelect().toString();
              String replacementString =
                  methodStr.substring(0, methodStr.lastIndexOf('.') + 1) + mst.getIdentifier();
              // note that we update the mst instead of the method invocation tree.
              fixBuilder.replace(mst, replacementString);
            } else {
              // otherwise, simply delete the entire argument
              fixBuilder.delete(e);
            }
          } else if (args.indexOf(e) == (args.size() - 1)) {
            ExpressionTree prevExpr = args.get(args.size() - 2);
            fixBuilder.delete(e);
            fixBuilder.replace(prevExpr, state.getSourceForNode(prevExpr), 0, 1);
          } else {
            fixBuilder.replace(e, "", 0, 1);
          }

          return buildDescription(tree).addFix(fixBuilder.build()).build();
        }
      }
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchMemberSelect(MemberSelectTree tree, VisitorState state) {

    /* for some expression a.b(..).c(..)...
        if one of the method invocations has a parameter new PluginFactory...
        this will remove the parameter if the invocation has more than one parameter
        when it has only one parameter, then it will remove the invocation itself.

        e.g., if the parameter count is one in b(..) above and corresponds to
        new PluginFactory, above expression will be refactored to a.c(..)

    */
    ExpressionTree et = tree.getExpression();
    if (et.getKind().equals(Kind.METHOD_INVOCATION)) {
      MethodInvocationTree mit = (MethodInvocationTree) et;
      return processArguments(mit, state, tree);
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchMethodInvocation(MethodInvocationTree tree, VisitorState state) {
    /* if this is part of a member select tree, let that matcher invoke the processing */
    if (state.getPath().getParentPath().getLeaf().getKind().equals(Kind.MEMBER_SELECT))
      return Description.NO_MATCH;
    return processArguments(tree, state, null);
  }

  @Override
  public Description matchClass(ClassTree tree, VisitorState state) {

    Symbol.ClassSymbol classSymbol = ASTHelpers.getSymbol(tree);
    String className = classSymbol.getSimpleName().toString();
    // Add comment in the class file that needs to be deleted
    if (className.equals(pluginFactory)) {
      return buildDescription(tree)
          .addFix(SuggestedFix.postfixWith(tree, PiranhaUtils.DELETE_REQUEST_COMMENT))
          .build();
    }

    // Update extends to remove reference to the removed plugin
    // bizarre but errorprone lists 'extends' clause under 'getImplementsClause'
    List<? extends Tree> l = tree.getImplementsClause();

    for (Tree t : l) {
      if (t.getKind().equals(Kind.MEMBER_SELECT)) {
        MemberSelectTree mst = (MemberSelectTree) t;
        if (mst.getExpression().toString().equals(pluginFactory)) {
          SuggestedFix.Builder fixBuilder = SuggestedFix.builder();
          if (l.size() == 1) {
            String treeString = tree.toString();
            int index = treeString.indexOf("extends");
            String deleteString = "__piranha_dummy";
            if (index <= 0) {
              index = treeString.indexOf("implements");
              deleteString = "implements " + t.toString();
            } else {
              deleteString = "extends " + t.toString();
            }

            if (index > 0) {
              String replacementString = treeString.replace(deleteString, "");
              fixBuilder.replace(tree, replacementString);
            }
          } else if (l.indexOf(t) == l.size() - 1) {
            Tree prevTree = l.get(l.size() - 2);
            fixBuilder.delete(t);
            fixBuilder.replace(prevTree, state.getSourceForNode(prevTree), 0, 1);
          } else {
            fixBuilder.replace(t, "", 0, 1);
          }

          return buildDescription(tree).addFix(fixBuilder.build()).build();
        }
      }
    }

    return Description.NO_MATCH;
  }

  @Override
  public Description matchVariable(VariableTree tree, VisitorState state) {
    Symbol sym = FindIdentifiers.findIdent(pluginName, state);
    // Check if this is the plugin definition and remove it.
    if (sym != null && sym.isEnum() && sym.equals(ASTHelpers.getSymbol(tree))) {
      String enumAsStr = state.getSourceForNode(state.getPath().getParentPath().getLeaf());
      String varAsStrWithComma = tree.getName().toString() + ",";
      if (enumAsStr.contains(varAsStrWithComma)) {
        return buildDescription(tree).addFix(SuggestedFix.replace(tree, "", 0, 1)).build();
      } else {
        Tree t = state.getPath().getParentPath().getLeaf();
        if (t.getKind().equals(Kind.ENUM)) {
          // for one entry, there are 2 elements including constructor
          // we delete the entire class for that.
          if (((ClassTree) t).getMembers().size() == 2) {
            return buildDescription(tree)
                .addFix(SuggestedFix.postfixWith(tree, PiranhaUtils.DELETE_REQUEST_COMMENT))
                .build();
          } else {
            return buildDescription(tree).addFix(SuggestedFix.delete(tree)).build();
          }
        }
      }
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchNewClass(NewClassTree tree, VisitorState state) {

    MethodTree mt = ASTHelpers.findEnclosingNode(state.getPath(), MethodTree.class);
    if (mt != null && mt.getName().toString().equals("createNewPlugin")) {
      ClassTree ct = ASTHelpers.findEnclosingNode(state.getPath(), ClassTree.class);
      if (ct != null && ct.getSimpleName().toString().equals(pluginFactory)) {
        helperClassName = tree.getIdentifier().toString();
        return buildDescription(tree)
            .addFix(
                SuggestedFix.postfixWith(
                    tree, PiranhaUtils.HELPER_CLASS + "=" + helperClassName + "="))
            .build();
      }
    }
    return Description.NO_MATCH;
  }

  @Override
  public Description matchImport(ImportTree importTree, VisitorState visitorState) {
    Tree importIdentifier = importTree.getQualifiedIdentifier();
    if (importIdentifier.getKind().equals(Kind.MEMBER_SELECT)) {
      MemberSelectTree memberSelectTree = (MemberSelectTree) importIdentifier;
      String mstId = memberSelectTree.getIdentifier().toString();
      // delete imports of plugin factory class and/or helper class files
      // if helper class is derived after this is processed, that may leave the helper class
      // import. we can either do a two pass or delete the helperClass import from the bash script
      if (mstId.endsWith(pluginFactory) || mstId.endsWith(helperClassName)) {
        return buildDescription(importTree)
            .addFix(SuggestedFix.replace(importTree, "", 0, 1))
            .build();
      }
    }
    return Description.NO_MATCH;
  }
}
